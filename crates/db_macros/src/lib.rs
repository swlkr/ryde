use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use sqlparser::ast::{
    AlterTableOperation, Assignment, Query, Select, SelectItem, SetExpr, TableFactor,
    TableWithJoins,
};
use sqlparser::parser::Parser;
use sqlparser::{ast::Statement, dialect::SQLiteDialect};
use std::collections::HashSet;
use syn::{
    parse_macro_input, punctuated::Punctuated, Expr, ExprAssign, ExprLit, Lit, Result, Token,
};

#[proc_macro]
pub fn db(input: TokenStream) -> TokenStream {
    let input =
        parse_macro_input!(input with Punctuated::<ExprAssign, Token![,]>::parse_terminated);
    match db_macro(input) {
        Ok(s) => s.to_token_stream().into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn db_macro(exprs: Punctuated<ExprAssign, Token![,]>) -> Result<TokenStream2> {
    let input = to_input(exprs);
    let output = to_output(input);
    let source = to_tokens(output);

    Ok(source)
}

fn to_input(exprs: Punctuated<ExprAssign, Token![,]>) -> Input {
    let defs = exprs.iter().filter_map(to_def).collect::<Vec<_>>();
    let columns = defs.iter().flat_map(columns).collect::<HashSet<Column>>();

    Input { defs, columns }
}

fn to_output(input: Input) -> Output {
    let stmts = input
        .defs
        .into_iter()
        .filter_map(|def| to_stmt(&input.columns, def))
        .collect();
    Output { stmts }
}

fn to_tokens(output: Output) -> TokenStream2 {
    let tokens: Vec<TokenStream2> = output.stmts.into_iter().map(stmt_tokens).collect();

    quote! { #(#tokens)* }
}

fn stmt_tokens(output: Stmt) -> TokenStream2 {
    match output {
        Stmt::ExecuteBatch { ident, sql } => quote! {
            pub async fn #ident() -> tokio_rusqlite::Result<()> {
                connection()
                    .await
                    .call(move |conn| conn.execute_batch(#sql).map_err(|err| err.into()))
                    .await?;

                Ok(())
            }
        },
        Stmt::Execute {
            ident,
            sql,
            in_cols,
        } => {
            let fn_args: Vec<TokenStream2> = in_cols.iter().map(fn_tokens).collect();
            let param_fields: Vec<TokenStream2> = in_cols.iter().map(param_tokens).collect();

            quote! {
                pub async fn #ident(#(#fn_args,)*) -> tokio_rusqlite::Result<usize> {
                    connection()
                        .await
                        .call(move |conn| {
                            let params = tokio_rusqlite::params![#(#param_fields,)*];
                            conn.execute(#sql, params).map_err(|err| err.into())
                        })
                        .await
                }
            }
        }
        Stmt::AggQuery {
            ident,
            sql,
            in_cols,
        } => {
            let fn_args: Vec<TokenStream2> = in_cols.iter().map(fn_tokens).collect();
            let param_fields: Vec<TokenStream2> = in_cols.iter().map(param_tokens).collect();

            quote! {
                 pub async fn #ident(#(#fn_args,)*) -> tokio_rusqlite::Result<i64> {
                    let result = connection()
                        .await
                        .call(move |conn| {
                            let mut stmt = conn.prepare(#sql)?;
                            let params = tokio_rusqlite::params![#(#param_fields,)*];
                            let rows = stmt.query_map(params, |row| row.get(0))?
                                .collect::<rusqlite::Result<Vec<_>>>();

                            match rows {
                                Ok(rows) => Ok(rows.last().cloned().expect("count(*) expected")),
                                Err(err) => Err(err.into()),
                            }
                        })
                        .await?;


                    Ok(result)
                }
            }
        }
        Stmt::Query {
            ident,
            sql,
            in_cols,
            out_cols,
            ret,
        } => {
            let struct_ident = struct_ident(&ident);
            let struct_fields: Vec<TokenStream2> = out_cols.iter().map(column_tokens).collect();
            let instance_fields: Vec<TokenStream2> = out_cols.iter().map(row_tokens).collect();
            let fn_args: Vec<TokenStream2> = in_cols.iter().map(fn_tokens).collect();
            let param_fields: Vec<TokenStream2> = in_cols.iter().map(param_tokens).collect();
            let (return_statement, return_type) = match ret {
                QueryReturn::Row => (
                    quote! {
                        match rows {
                            Ok(rows) => Ok(rows.last().cloned()),
                            Err(err) => Err(err.into()),
                        }
                    },
                    quote! { Option<#struct_ident> },
                ),
                QueryReturn::Rows => (
                    quote! {
                        rows.map_err(|err| err.into())
                    },
                    quote! { Vec<#struct_ident> },
                ),
            };

            quote! {
                #[derive(Default, Debug, Deserialize, Serialize, Clone)]
                #[serde(crate = "crate::serde")]
                pub struct #struct_ident {
                    #(#struct_fields,)*
                }

                impl #struct_ident {
                    pub fn new(row: &tokio_rusqlite::Row<'_>) -> rusqlite::Result<Self> {
                        Ok(Self { #(#instance_fields,)* ..Default::default() })
                    }
                }

                 pub async fn #ident(#(#fn_args,)*) -> tokio_rusqlite::Result<#return_type> {
                    connection()
                        .await
                        .call(move |conn| {
                            let mut stmt = conn.prepare(#sql)?;
                            let params = tokio_rusqlite::params![#(#param_fields,)*];
                            let rows = stmt
                                .query_map(params, |row| #struct_ident::new(row))?
                                .collect::<rusqlite::Result<Vec<#struct_ident>>>();
                            #return_statement
                        })
                        .await
                }
            }
        }
    }
}

#[derive(Debug)]
enum QueryReturn {
    Row,
    Rows,
}

fn to_stmt(
    db_columns: &HashSet<Column>,
    Def {
        statements,
        ident,
        sql,
    }: Def,
) -> Option<Stmt> {
    // last one is the only one that returns anything?
    match statements.last() {
        Some(stmt) => match stmt {
            Statement::Insert {
                table_name,
                columns,
                returning,
                source,
                ..
            } => insert_stmt(
                db_columns,
                ident,
                sql,
                table_name.to_string(),
                columns,
                returning,
                source,
            ),
            Statement::Update {
                table,
                assignments,
                from,
                selection,
                returning,
            } => update_stmt(
                db_columns,
                ident,
                sql,
                table,
                assignments,
                from,
                selection,
                returning,
            ),
            Statement::Delete {
                from,
                selection,
                returning,
                ..
            } => delete_stmt(db_columns, ident, sql, from, selection, returning),
            Statement::Query(q) => {
                let Query { body, limit, .. } = &**q;
                query_stmt(db_columns, ident, sql, body, limit.as_ref())
            }
            _ => Some(Stmt::ExecuteBatch { ident, sql }),
        },
        _ => None,
    }
}

fn query_stmt(
    db_cols: &HashSet<Column>,
    ident: Ident,
    sql: String,
    body: &SetExpr,
    limit: Option<&sqlparser::ast::Expr>,
) -> Option<Stmt> {
    let SetExpr::Select(select) = body else {
        return None;
    };
    let Select {
        projection,
        selection,
        ..
    } = &**select;
    let in_cols = match selection {
        Some(expr) => columns_from_expr(&db_cols, expr),
        None => vec![],
    };
    let out_cols = projection
        .iter()
        .flat_map(|si| columns_from_select_item(&db_cols, si))
        .collect::<Vec<_>>();
    let ret = match limit {
        Some(sqlparser::ast::Expr::Value(sqlparser::ast::Value::Number(number, _))) => {
            match number.as_str() {
                "1" => QueryReturn::Row,
                _ => QueryReturn::Rows,
            }
        }
        _ => QueryReturn::Rows,
    };
    // one column and it's count(*)
    let ret = match out_cols[..] {
        [Column {
            ref column_type, ..
        }] => match column_type {
            ColumnType::Aggregate => {
                return Some(Stmt::AggQuery {
                    ident,
                    sql,
                    in_cols,
                })
            }
            ColumnType::Column => ret,
        },
        _ => ret,
    };

    Some(Stmt::Query {
        ident,
        sql,
        in_cols,
        out_cols,
        ret,
    })
}

fn update_stmt(
    db_cols: &HashSet<Column>,
    ident: Ident,
    sql: String,
    table: &TableWithJoins,
    assignments: &[Assignment],
    _from: &Option<TableWithJoins>,
    selection: &Option<sqlparser::ast::Expr>,
    returning: &Option<Vec<SelectItem>>,
) -> Option<Stmt> {
    let table_names = table_names(table);
    let table_name = table_names.get(0);
    let table_name = match table_name {
        Some(t) => t,
        None => panic!("update needs a table name in {}", &ident),
    };

    let table_columns = db_cols
        .iter()
        .filter(|c| &c.table_name == table_name)
        .map(|c| c.clone())
        .collect::<HashSet<_>>();
    let out_cols = match returning {
        Some(si) => si
            .iter()
            .flat_map(|si| columns_from_select_item(&table_columns, si))
            .collect::<Vec<_>>(),
        None => vec![],
    };
    let mut in_cols = assignments
        .iter()
        .filter_map(|a| match a.value {
            sqlparser::ast::Expr::Value(sqlparser::ast::Value::Placeholder(_)) => Some(&a.id),
            _ => None,
        })
        .flat_map(|c| columns_from_idents(&table_columns, c))
        .collect::<Vec<_>>();
    in_cols.extend(match selection {
        Some(expr) => columns_from_expr(&table_columns, expr),
        None => vec![],
    });

    match returning {
        Some(_) => Some(Stmt::Query {
            ident,
            sql,
            in_cols,
            out_cols,
            ret: QueryReturn::Row,
        }),
        None => Some(Stmt::Execute {
            ident,
            sql,
            in_cols,
        }),
    }
}

fn to_def(ExprAssign { left, right, .. }: &ExprAssign) -> Option<Def> {
    let Expr::Path(syn::ExprPath { path, .. }) = &**left else {
        return None;
    };
    let Some(ident) = path.get_ident() else {
        return None;
    };
    let Expr::Lit(ExprLit {
        lit: Lit::Str(lit_str),
        ..
    }) = &**right
    else {
        return None;
    };
    let sql = lit_str.value();
    let statements = match Parser::parse_sql(&SQLiteDialect {}, &sql) {
        Ok(ast) => ast,
        Err(err) => {
            // TODO: better error handling
            panic!("{}", err);
        }
    };

    Some(Def {
        ident: ident.clone(),
        sql,
        statements,
    })
}

fn columns(Def { statements, .. }: &Def) -> HashSet<Column> {
    statements
        .iter()
        .filter_map(|statement| match statement {
            Statement::CreateTable { name, columns, .. } => {
                let name = name.to_string();
                let columns = columns
                    .iter()
                    .map(|c| column(Some(&name), c))
                    .collect::<Vec<_>>();

                Some(columns)
            }
            Statement::AlterTable {
                name, operations, ..
            } => {
                let name = name.to_string();
                let columns = operations
                    .iter()
                    .filter_map(|op| match op {
                        AlterTableOperation::AddColumn { column_def, .. } => {
                            Some(column(Some(&name), column_def))
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();

                Some(columns)
            }
            _ => None,
        })
        .flat_map(|c| c)
        .collect()
}

fn table_names(table: &TableWithJoins) -> Vec<String> {
    let mut results = table_names_from(&table.relation);
    results.extend(
        table
            .joins
            .iter()
            .flat_map(|j| table_names_from(&j.relation)),
    );

    results
}

fn table_names_from(relation: &TableFactor) -> Vec<String> {
    match relation {
        sqlparser::ast::TableFactor::Table { name, .. } => vec![name.to_string()],
        sqlparser::ast::TableFactor::NestedJoin {
            table_with_joins, ..
        } => table_names(&table_with_joins),
        _ => vec![],
    }
}

fn struct_ident(ident: &Ident) -> Ident {
    syn::Ident::new(&snake_to_pascal(ident.to_string()), ident.span())
}

fn insert_stmt(
    db_cols: &HashSet<Column>,
    ident: Ident,
    sql: String,
    table_name: String,
    columns: &Vec<sqlparser::ast::Ident>,
    returning: &Option<Vec<SelectItem>>,
    source: &Option<Box<Query>>,
) -> Option<Stmt> {
    // nice little compile time validation
    // check insert into count matches placeholder count
    let placeholder_count = match source.as_deref() {
        Some(Query { body, .. }) => match &**body {
            SetExpr::Values(value) => value
                .rows
                .iter()
                .flatten()
                .filter(|expr| match expr {
                    sqlparser::ast::Expr::Value(sqlparser::ast::Value::Placeholder(_)) => true,
                    _ => false,
                })
                .count(),
            _ => 0,
        },
        None => 0,
    };
    let input_col_names = columns.iter().map(|c| c.to_string()).collect::<Vec<_>>();
    if placeholder_count != input_col_names.len() {
        panic!("{} placeholder count doesn't match insert into", ident);
    }

    let table_columns = db_cols
        .iter()
        .filter(|c| c.table_name == table_name)
        .map(|c| c.clone())
        .collect::<HashSet<_>>();

    // check insert into matches table columns
    let table_column_names = table_columns
        .iter()
        .map(|c| c.name.clone())
        .collect::<Vec<_>>();
    for n in input_col_names {
        if !table_column_names.contains(&n) {
            panic!("column {} does not exist in table {}", n, table_name);
        }
    }

    let in_cols: Vec<Column> = columns_from_idents(&table_columns, columns);

    match returning {
        Some(_) => {
            let out_cols: Vec<Column> = match returning {
                Some(si) => si
                    .iter()
                    .flat_map(|si| columns_from_select_item(&table_columns, si))
                    .collect(),
                None => vec![],
            };
            Some(Stmt::Query {
                ident,
                sql,
                in_cols,
                out_cols,
                ret: QueryReturn::Row,
            })
        }
        None => Some(Stmt::Execute {
            ident,
            sql,
            in_cols,
        }),
    }
}

fn delete_stmt(
    db_cols: &HashSet<Column>,
    ident: Ident,
    sql: String,
    from: &Vec<TableWithJoins>,
    selection: &Option<sqlparser::ast::Expr>,
    returning: &Option<Vec<SelectItem>>,
) -> Option<Stmt> {
    let table_names = from.iter().flat_map(|f| table_names(f)).collect::<Vec<_>>();
    let table_name = match table_names.first() {
        Some(t) => t.to_string(),
        None => panic!("delete expects table name {}", &ident),
    };
    let table_columns = db_cols
        .iter()
        .filter(|c| c.table_name == table_name)
        .map(|c| c.clone())
        .collect::<HashSet<_>>();
    let in_cols = match selection {
        Some(expr) => columns_from_expr(&table_columns, expr),
        None => vec![],
    };
    let out_cols = match returning {
        Some(si) => si
            .iter()
            .flat_map(|si| columns_from_select_item(&table_columns, si))
            .collect::<Vec<_>>(),
        None => vec![],
    };

    match returning {
        Some(_) => Some(Stmt::Query {
            ident,
            sql,
            in_cols,
            out_cols,
            ret: QueryReturn::Row,
        }),
        _ => Some(Stmt::Execute {
            ident,
            sql,
            in_cols,
        }),
    }
}

fn snake_to_pascal(input: String) -> String {
    input
        .split("_")
        .filter(|x| !x.is_empty())
        .map(|x| {
            let mut chars = x.chars();
            format!("{}{}", chars.nth(0).unwrap().to_uppercase(), chars.as_str())
        })
        .collect::<String>()
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
enum ColumnType {
    Aggregate,
    #[default]
    Column,
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Hash)]
struct Column {
    name: String,
    full_name: String,
    table_name: String,
    column_type: ColumnType,
    data_type: DataType,
}

fn columns_from_idents(
    table_columns: &HashSet<Column>,
    column_names: &Vec<sqlparser::ast::Ident>,
) -> Vec<Column> {
    column_names
        .iter()
        .filter_map(|ident| {
            table_columns
                .iter()
                .find(|c| c.name == ident.value)
                .cloned()
        })
        .collect::<Vec<_>>()
}

fn columns_from_select_item(
    table_columns: &HashSet<Column>,
    select_item: &sqlparser::ast::SelectItem,
) -> HashSet<Column> {
    match select_item {
        sqlparser::ast::SelectItem::UnnamedExpr(expr) => columns_from_expr(&table_columns, expr)
            .into_iter()
            .collect::<HashSet<_>>(),
        sqlparser::ast::SelectItem::ExprWithAlias { .. } => todo!(),
        sqlparser::ast::SelectItem::QualifiedWildcard(obj_name, _) => table_columns
            .iter()
            .filter(|c| c.table_name == obj_name.to_string())
            .map(|c| c.clone())
            .collect::<HashSet<_>>(),
        sqlparser::ast::SelectItem::Wildcard(_) => panic!("unqualified * not supported yet"),
    }
}

fn columns_from_expr(table_columns: &HashSet<Column>, expr: &sqlparser::ast::Expr) -> Vec<Column> {
    match expr {
        sqlparser::ast::Expr::Identifier(ident) => {
            match table_columns.iter().find(|c| c.name == ident.to_string()) {
                Some(c) => vec![c.clone()],
                None => panic!("column {} does not exist", ident.to_string()),
            }
        }
        sqlparser::ast::Expr::CompoundIdentifier(idents) => {
            let name = idents
                .into_iter()
                .map(|ident| ident.value.clone())
                .collect::<Vec<_>>()
                .join(".");
            match table_columns.iter().find(|c| c.full_name == name) {
                Some(c) => vec![c.clone()],
                None => vec![],
            }
        }
        sqlparser::ast::Expr::Wildcard => {
            panic!("unqualified * not supported yet");
            // table_columns.iter().map(|c| c.clone()).collect::<Vec<_>>()
        }
        sqlparser::ast::Expr::QualifiedWildcard(obj_name) => table_columns
            .iter()
            .filter(|c| c.table_name == obj_name.to_string())
            .map(|c| c.clone())
            .collect::<Vec<_>>(),
        sqlparser::ast::Expr::BinaryOp { left, right, .. } => vec![
            columns_from_expr(&table_columns, left),
            columns_from_expr(&table_columns, right),
        ]
        .into_iter()
        .flat_map(|c| c)
        .collect::<Vec<_>>(),
        sqlparser::ast::Expr::Value(_) => vec![],
        sqlparser::ast::Expr::Function(sqlparser::ast::Function { name, args, .. }) => {
            match name.to_string().as_str() {
                "count" => {
                    let name = args
                        .iter()
                        .map(|fa| match fa {
                            sqlparser::ast::FunctionArg::Unnamed(fa_expr) => match fa_expr {
                                sqlparser::ast::FunctionArgExpr::Expr(expr) => {
                                    columns_from_expr(&table_columns, expr)
                                        .iter()
                                        .map(|c| c.full_name.clone())
                                        .collect::<Vec<_>>()
                                        .join("")
                                }
                                sqlparser::ast::FunctionArgExpr::QualifiedWildcard(table_name) => {
                                    format!("{}({}.*)", name, table_name)
                                }
                                sqlparser::ast::FunctionArgExpr::Wildcard => format!("{}(*)", name),
                            },
                            _ => todo!(),
                        })
                        .collect::<Vec<_>>()
                        .join("");
                    vec![Column {
                        name: name.clone(),
                        full_name: name,
                        table_name: "".into(),
                        data_type: DataType::Integer,
                        column_type: ColumnType::Aggregate,
                    }]
                }
                _ => todo!("what"),
            }
        }
        _ => todo!(),
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Hash)]
enum DataType {
    Integer,
    Real,
    Text,
    #[default]
    Blob,
    Any,
    Null(Box<DataType>),
}

fn full_column_name(table_name: Option<&String>, column_name: String) -> String {
    match table_name {
        Some(table_name) => format!("{}.{}", table_name, column_name),
        None => column_name,
    }
}

fn column(table_name: Option<&String>, value: &sqlparser::ast::ColumnDef) -> Column {
    let name = value.name.to_string();
    let full_name = full_column_name(table_name, name.clone());
    let inner_data_type = match &value.data_type {
        sqlparser::ast::DataType::Blob(_) => DataType::Blob,
        sqlparser::ast::DataType::Integer(_) => DataType::Integer,
        sqlparser::ast::DataType::Int(_) => DataType::Integer,
        sqlparser::ast::DataType::Real => DataType::Real,
        sqlparser::ast::DataType::Text => DataType::Text,
        _ => DataType::Any,
    };
    let data_type = match not_null(&inner_data_type, &value.options) {
        true => inner_data_type,
        false => DataType::Null(inner_data_type.into()),
    };
    let table_name = table_name.unwrap_or(&"".into()).clone();

    Column {
        name,
        full_name,
        table_name,
        data_type,
        ..Default::default()
    }
}

fn not_null(data_type: &DataType, value: &Vec<sqlparser::ast::ColumnOptionDef>) -> bool {
    value.iter().any(|opt| match opt.option {
        sqlparser::ast::ColumnOption::NotNull => true,
        sqlparser::ast::ColumnOption::Unique { is_primary, .. } => {
            is_primary && data_type == &DataType::Integer
        }
        _ => false,
    })
}

fn data_type_tokens(data_type: &DataType) -> TokenStream2 {
    match data_type {
        DataType::Integer => quote!(i64),
        DataType::Real => quote!(f64),
        DataType::Text => quote!(String),
        DataType::Blob => quote!(Vec<u8>),
        DataType::Any => quote!(Vec<u8>),
        DataType::Null(null) => {
            let tokens = data_type_tokens(null);
            quote!(Option<#tokens>)
        }
    }
}

fn column_tokens(column: &Column) -> TokenStream2 {
    let data_type = data_type_tokens(&column.data_type);
    let name = syn::Ident::new(&column.name, proc_macro2::Span::call_site());

    quote!(#name: #data_type)
}

fn param_tokens(column: &Column) -> TokenStream2 {
    let name = syn::Ident::new(&column.name, proc_macro2::Span::call_site());
    quote!(#name)
}

fn row_tokens(column: &Column) -> TokenStream2 {
    let lit_str = &column.name;
    let ident = syn::Ident::new(&lit_str, proc_macro2::Span::call_site());

    quote!(#ident: row.get(#lit_str)?)
}

fn fn_tokens(column: &Column) -> TokenStream2 {
    let lit_str = &column.name;
    let ident = syn::Ident::new(&lit_str, proc_macro2::Span::call_site());
    let fn_type = fn_type(&column.data_type);

    quote!(#ident: #fn_type)
}

fn fn_type(data_type: &DataType) -> TokenStream2 {
    match data_type {
        DataType::Integer => quote!(i64),
        DataType::Real => quote!(f64),
        DataType::Text => quote!(String),
        DataType::Blob => quote!(Vec<u8>),
        DataType::Any => quote!(Vec<u8>),
        DataType::Null(dt) => {
            let dt = fn_type(&*dt);
            quote!(Option<#dt>)
        }
    }
}

#[derive(Debug)]
struct Def {
    ident: Ident,
    sql: String,
    statements: Vec<Statement>,
}

#[derive(Debug)]
struct Input {
    defs: Vec<Def>,
    columns: HashSet<Column>,
}

#[derive(Debug)]
struct Output {
    stmts: Vec<Stmt>,
}

#[derive(Debug)]
enum Stmt {
    ExecuteBatch {
        ident: Ident,
        sql: String,
    },
    Execute {
        ident: Ident,
        sql: String,
        in_cols: Vec<Column>,
    },
    AggQuery {
        ident: Ident,
        sql: String,
        in_cols: Vec<Column>,
    },
    Query {
        ident: Ident,
        sql: String,
        in_cols: Vec<Column>,
        out_cols: Vec<Column>,
        ret: QueryReturn,
    },
}
