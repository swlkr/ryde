use std::collections::HashSet;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use rusqlite::Connection;
use sqlparser::parser::Parser;
use sqlparser::{ast::Statement, dialect::SQLiteDialect};
use syn::{
    parse_macro_input, punctuated::Punctuated, Expr, ExprLit, ExprTuple, Lit, Result, Token,
};

#[proc_macro]
pub fn db(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input with Punctuated::<Expr, Token![,]>::parse_terminated);
    match db_macro(input) {
        Ok(s) => s.to_token_stream().into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn db_macro(input: Punctuated<Expr, Token![,]>) -> Result<TokenStream2> {
    let database_url = std::env::var("DATABASE_URL").unwrap_or("db.sqlite3".into());
    let connection = connection(database_url);
    let mut tables: Vec<Table> = vec![];

    let source = input
        .iter()
        .map(|expr| {
            match expr {
                Expr::Lit(ExprLit { lit, .. }) => match lit {
                    Lit::Str(lit_str) => {
                        let sql = lit_str.value();
                        if let Err(_) = connection.execute_batch(&sql) {
                            panic!("Failed to execute {}", &sql);
                        }
                        let ast = match Parser::parse_sql(&SQLiteDialect {}, &sql) {
                            Ok(ast) => ast,
                            Err(err) => {
                                // TODO: better error handling
                                panic!("{}", err);
                            }
                        };
                        tables.push(table(&ast));
                        quote!()
                    }
                    _ => todo!(),
                },
                Expr::Tuple(ExprTuple { elems, .. }) => {
                    let Some(Expr::Lit(ExprLit { lit, .. })) = elems.last() else {
                        return quote!();
                    };
                    let Lit::Str(lit_str) = lit else {
                        return quote!();
                    };
                    let sql = lit_str.value();
                    let ast = match Parser::parse_sql(&SQLiteDialect {}, &sql) {
                        Ok(ast) => ast,
                        Err(err) => {
                            // TODO: better error handling
                            panic!("{}", err);
                        }
                    };

                    if let Some(Expr::Path(path)) = elems.first() {
                        if let Some(ident) = path.path.get_ident() {
                            let is_execute = is_execute(&ast);
                            let columns = columns(&ast);
                            let columns = reify_columns(&tables, &columns);
                            let inputs = columns.iter().filter_map(input_column).collect::<Vec<_>>();
                            let outputs = columns.iter().filter_map(output_column).collect::<Vec<_>>();
                            let mut columns = columns.iter().map(column_from_col).collect::<Vec<_>>();
                            let columns = unique_columns(&mut columns);
                            let column_fields =
                                columns.iter().map(|column: &&Column| column_tokens(*column)).collect::<Vec<_>>();
                            let param_fields = inputs.iter().map(|column| param_tokens(*column)).collect::<Vec<_>>();
                            let row_fields = outputs.iter().map(|column: &&Column| row_tokens(*column)).collect::<Vec<_>>();
                            let fn_args = inputs.iter().map(|column: &&Column| fn_tokens(*column)).collect::<Vec<_>>();
                            let struct_fields_tokens= inputs.iter().map(|column: &&Column| struct_fields_tokens(*column)).collect::<Vec<_>>();
                            let struct_ident = syn::Ident::new(&snake_to_pascal(ident.to_string()), ident.span());
                            let has_limit = limit_one(&ast);
                            let (query_fn, return_value) = match has_limit {
                                true => (quote! { query_one }, quote! { Option<#struct_ident> }),
                                false => (quote! { query }, quote! { Vec<#struct_ident> })
                            };
                            if column_fields.is_empty() {
                                quote!(
                                    #[derive(Default, Debug, Deserialize, Serialize, Clone)]
                                    #[serde(crate = "crate::serde")]
                                    struct #struct_ident;

                                    async fn #ident() -> tokio_rusqlite::Result<#return_value> {
                                        ryde_db::#query_fn(#struct_ident {}).await
                                    }
                                )
                            } else {
                                quote!(
                                    #[derive(Default, Debug, Deserialize, Serialize, Clone)]
                                    #[serde(crate = "crate::serde")]
                                    struct #struct_ident {
                                        #(#column_fields,)*
                                    }

                                    impl ryde_db::Query for #struct_ident {
                                        fn sql() -> &'static str {
                                            #sql
                                        }

                                        fn is_execute() -> bool {
                                            #is_execute
                                        }

                                        fn params(&self) -> Vec<tokio_rusqlite::types::Value> {
                                            vec![#(#param_fields,)*]
                                        }

                                        fn new(row: &tokio_rusqlite::Row<'_>) -> rusqlite::Result<Self> {
                                            Ok(Self { #(#row_fields,)* ..Default::default() })
                                        }
                                    }

                                    async fn #ident(#(#fn_args,)*) -> tokio_rusqlite::Result<#return_value> {
                                        ryde_db::#query_fn(#struct_ident {
                                            #(#struct_fields_tokens,)*
                                            ..Default::default()
                                        }).await
                                    }
                                )
                            }
                        } else {
                            quote!()
                        }
                    } else {
                        quote!()
                    }
                }
                _ => todo!(),
            }
        })
        .collect::<Vec<_>>();

    Ok(quote! {
        #(#source)*
    })
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

fn unique_columns<'a>(columns: &'a mut Vec<&'a Column>) -> &'a Vec<&'a Column> {
    let mut seen = HashSet::new();
    columns.retain(|item| seen.insert(*item));
    columns
}

fn connection(url: String) -> Connection {
    let connection = Connection::open(url).expect("Failed to connect to db");
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;",
        )
        .expect("Failed to connect to db");
    connection
}

#[derive(Debug)]
struct Table {
    columns: Vec<Column>,
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Hash)]
struct Column {
    name: String,
    data_type: DataType,
}

impl From<&Col> for Column {
    fn from(value: &Col) -> Self {
        match value {
            Col::Input(c) => c.clone(),
            Col::Output(c) => c.clone(),
        }
    }
}

fn reify_columns(tables: &Vec<Table>, columns: &Vec<Col>) -> Vec<Col> {
    let schema_columns: Vec<&Column> = tables.iter().flat_map(|table| &table.columns).collect();

    if columns.len() == 1 && Column::from(columns.last().unwrap()).name == "*" {
        let col = columns.last().unwrap();
        match col {
            Col::Input(_) => schema_columns
                .into_iter()
                .map(|sc| Col::Input(sc.clone()))
                .collect(),
            Col::Output(_) => schema_columns
                .into_iter()
                .map(|sc| Col::Output(sc.clone()))
                .collect(),
        }
    } else {
        columns
            .iter()
            .flat_map(|c| {
                let col = match c {
                    Col::Input(col) => col,
                    Col::Output(col) => col,
                };

                let data_type = match schema_columns.iter().find(|c1| c1.name == col.name) {
                    Some(column) => column.data_type.clone(),
                    None => DataType::Blob,
                };

                if col.name == "*" {
                    schema_columns
                        .iter()
                        .map(|col| {
                            let col = *col;
                            match c {
                                Col::Input(_) => Col::Input(col.clone()),
                                Col::Output(_) => Col::Output(col.clone()),
                            }
                        })
                        .collect()
                } else {
                    let output = Column {
                        name: col.name.clone(),
                        data_type,
                    };

                    vec![match c {
                        Col::Input(_) => Col::Input(output),
                        Col::Output(_) => Col::Output(output),
                    }]
                }
            })
            .collect()
    }
}

fn columns_from_idents(columns: &Vec<sqlparser::ast::Ident>) -> Vec<Column> {
    columns
        .iter()
        .map(|c| Column {
            name: c.value.clone(),
            ..Default::default()
        })
        .collect()
}

fn column_from_select_item(select_item: &sqlparser::ast::SelectItem) -> Column {
    match select_item {
        sqlparser::ast::SelectItem::UnnamedExpr(expr) => column_from_expr(expr),
        sqlparser::ast::SelectItem::ExprWithAlias { .. } => todo!(),
        sqlparser::ast::SelectItem::QualifiedWildcard(_, _)
        | sqlparser::ast::SelectItem::Wildcard(_) => Column {
            name: "*".into(),
            ..Default::default()
        },
    }
}

fn column_from_expr(expr: &sqlparser::ast::Expr) -> Column {
    match expr {
        sqlparser::ast::Expr::Identifier(ident) => Column {
            name: ident.value.clone(),
            ..Default::default()
        },
        sqlparser::ast::Expr::CompoundIdentifier(_) => todo!(),
        sqlparser::ast::Expr::Wildcard | sqlparser::ast::Expr::QualifiedWildcard(_) => Column {
            name: "*".into(),
            ..Default::default()
        },
        sqlparser::ast::Expr::BinaryOp { left, .. } => column_from_expr(left),
        _ => todo!(),
    }
}

#[derive(Debug)]
enum Col {
    Input(Column),
    Output(Column),
}

fn columns_from_query(query: &sqlparser::ast::Query) -> Vec<Col> {
    match &*query.body {
        sqlparser::ast::SetExpr::Select(select) => {
            let mut v = select
                .projection
                .iter()
                .map(column_from_select_item)
                .map(|c| Col::Output(c))
                .collect::<Vec<_>>();
            if let Some(ref expr) = select.selection {
                v.push(Col::Input(column_from_expr(&expr)));
            }
            v
        }
        sqlparser::ast::SetExpr::Query(query) => columns_from_query(query),
        sqlparser::ast::SetExpr::SetOperation { .. } => todo!(),
        sqlparser::ast::SetExpr::Values(_) => todo!(),
        sqlparser::ast::SetExpr::Insert(_) => todo!(),
        sqlparser::ast::SetExpr::Update(_) => unreachable!(),
        sqlparser::ast::SetExpr::Table(_) => todo!(),
    }
}

fn columns(ast: &Vec<Statement>) -> Vec<Col> {
    ast.iter()
        .flat_map(|statement| match statement {
            Statement::Insert {
                columns, returning, ..
            } => {
                let mut cols = if let Some(returning) = returning {
                    returning
                        .iter()
                        .map(|si| column_from_select_item(si))
                        .map(|c| Col::Output(c))
                        .collect()
                } else {
                    vec![]
                };
                cols.extend(
                    columns_from_idents(columns)
                        .into_iter()
                        .map(|c| Col::Input(c)),
                );
                cols
            }
            Statement::Query(query) => columns_from_query(&query),
            Statement::Update {
                assignments,
                selection,
                returning,
                ..
            } => {
                let mut cols = assignments
                    .iter()
                    .map(|a| {
                        let name = if let Some(id) = a.id.last() {
                            id.to_string()
                        } else {
                            "".into()
                        };
                        Col::Input(Column {
                            name,
                            ..Default::default()
                        })
                    })
                    .collect::<Vec<Col>>();

                if let Some(selection) = selection {
                    cols.push(Col::Input(column_from_expr(selection)));
                }

                if let Some(returning) = returning {
                    cols.extend(
                        returning
                            .iter()
                            .map(|si| column_from_select_item(si))
                            .map(|c| Col::Output(c)),
                    );
                }

                cols
            }
            Statement::Delete {
                selection,
                returning,
                ..
            } => {
                let mut cols = if let Some(selection) = selection {
                    vec![Col::Input(column_from_expr(selection))]
                } else {
                    vec![]
                };

                if let Some(returning) = returning {
                    cols.extend(
                        returning
                            .iter()
                            .map(|si| column_from_select_item(si))
                            .map(|c| Col::Output(c)),
                    );
                }

                cols
            }
            _ => vec![],
        })
        .collect::<Vec<_>>()
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

fn column(value: &sqlparser::ast::ColumnDef) -> Column {
    let name = value.name.to_string();
    let data_type = match &value.data_type {
        sqlparser::ast::DataType::Blob(_) => DataType::Blob,
        sqlparser::ast::DataType::Integer(_) => DataType::Integer,
        sqlparser::ast::DataType::Int(_) => DataType::Integer,
        sqlparser::ast::DataType::Real => DataType::Real,
        sqlparser::ast::DataType::Text => DataType::Text,
        _ => DataType::Any,
    };
    let data_type = if is_null(&data_type, &value.options) {
        DataType::Null(data_type.into())
    } else {
        data_type
    };

    Column { name, data_type }
}

fn is_null(data_type: &DataType, value: &Vec<sqlparser::ast::ColumnOptionDef>) -> bool {
    value
        .iter()
        .filter_map(|opt| match opt.option {
            sqlparser::ast::ColumnOption::NotNull => Some(()),
            sqlparser::ast::ColumnOption::Unique { is_primary, .. } => {
                if is_primary == true && data_type == &DataType::Integer {
                    Some(())
                } else {
                    None
                }
            }
            _ => Some(()),
        })
        .count()
        == 0
}

fn table(ast: &Vec<Statement>) -> Table {
    let columns = ast
        .iter()
        .flat_map(|statement| match statement {
            Statement::CreateTable { columns, .. } => {
                columns.iter().map(column).collect::<Vec<_>>()
            }
            Statement::AlterTable { .. } => todo!(),
            _ => vec![],
        })
        .collect();

    Table { columns }
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

fn column_from_col(col: &Col) -> &Column {
    match col {
        Col::Input(col) => col,
        Col::Output(col) => col,
    }
}

fn column_tokens(column: &Column) -> TokenStream2 {
    let data_type = data_type_tokens(&column.data_type);
    let name = syn::Ident::new(&column.name, proc_macro2::Span::call_site());

    quote!(#name: #data_type)
}

fn param_tokens(column: &Column) -> TokenStream2 {
    let name = syn::Ident::new(&column.name, proc_macro2::Span::call_site());

    match &column.data_type {
        DataType::Integer => quote!(tokio_rusqlite::types::Value::Integer(self.#name)),
        DataType::Real => quote!(tokio_rusqlite::types::Value::Real(self.#name)),
        DataType::Text => quote!(tokio_rusqlite::types::Value::Text(self.#name.clone())),
        DataType::Blob => quote!(tokio_rusqlite::types::Value::Blob(self.#name.clone())),
        DataType::Any => quote!(tokio_rusqlite::types::Value::Blob(self.#name.clone())),
        DataType::Null(_) => quote!(tokio_rusqlite::types::Value::Null),
    }
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

fn struct_fields_tokens(column: &Column) -> TokenStream2 {
    let lit_str = &column.name;
    let ident = syn::Ident::new(&lit_str, proc_macro2::Span::call_site());

    quote!(#ident)
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

fn is_execute(ast: &Vec<Statement>) -> bool {
    ast.iter()
        .find(|statement| match statement {
            // TODO: check returning and return false here instead?
            Statement::Insert { .. } | Statement::Update { .. } | Statement::Delete { .. } => true,
            _ => false,
        })
        .is_some()
}

fn limit_one(ast: &Vec<Statement>) -> bool {
    ast.iter()
        .find(|statement| match statement {
            Statement::Query(query) => match &query.limit {
                Some(expr) => match expr {
                    sqlparser::ast::Expr::Value(value) => match value {
                        sqlparser::ast::Value::Number(x, _) => match x.parse::<i64>() {
                            Ok(x) => x == 1,
                            Err(_) => false,
                        },
                        _ => false,
                    },
                    _ => false,
                },
                None => false,
            },
            _ => false,
        })
        .is_some()
}

fn output_column(col: &Col) -> Option<&Column> {
    match col {
        Col::Input(_) => None,
        Col::Output(c) => Some(c),
    }
}

fn input_column(col: &Col) -> Option<&Column> {
    match col {
        Col::Input(input) => Some(input),
        Col::Output(_) => None,
    }
}
