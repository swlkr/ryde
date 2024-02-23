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
                    // TODO: returning * as third struct
                    // 2 elements
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
                            columns.dedup();
                            let column_fields =
                                columns.iter().map(|column: &&Column| column_tokens(*column)).collect::<Vec<_>>();
                            let param_fields = inputs.iter().map(|column| param_tokens(*column)).collect::<Vec<_>>();
                            let row_fields = outputs.iter().map(|column: &&Column| row_tokens(*column)).collect::<Vec<_>>();
                            if column_fields.is_empty() {
                                quote!(struct #ident;)
                            } else {
                                quote!(
                                    #[derive(Default, Debug)]
                                    struct #ident {
                                        #(#column_fields,)*
                                    }

                                    impl db::Query for #ident {
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

#[derive(Default, Debug, PartialEq, Eq)]
struct Column {
    name: String,
    data_type: DataType,
}

// fn table_name(table_factor: &sqlparser::ast::TableFactor) -> String {
//     match table_factor {
//         sqlparser::ast::TableFactor::Table { name, .. } => name.0.last().unwrap().value.to_owned(),
//         _ => todo!(),
//     }
// }

// fn table_names(ast: &Vec<Statement>) -> Vec<String> {
//     ast.iter()
//         .flat_map(|statement| match statement {
//             Statement::Insert { table_name, .. } => {
//                 vec![table_name.to_string()]
//             }
//             Statement::Query(query) => match &*query.body {
//                 sqlparser::ast::SetExpr::Select(select) => select
//                     .from
//                     .iter()
//                     .flat_map(|f| {
//                         let mut table_names = vec![table_name(&f.relation)];
//                         table_names.extend(f.joins.iter().map(|j| table_name(&j.relation)));

//                         table_names
//                     })
//                     .collect::<Vec<_>>(),
//                 _ => todo!(),
//             },
//             _ => todo!(),
//         })
//         .collect::<Vec<_>>()
// }

fn reify_columns(tables: &Vec<Table>, columns: &Vec<Col>) -> Vec<Col> {
    let schema_columns: Vec<&Column> = tables.iter().flat_map(|table| &table.columns).collect();

    columns
        .iter()
        .map(|c| {
            let col = match c {
                Col::Input(col) => col,
                Col::Output(col) => col,
            };

            let data_type = match schema_columns.iter().find(|c1| c1.name == col.name) {
                Some(column) => column.data_type.clone(),
                None => DataType::Blob,
            };

            let output = Column {
                name: col.name.clone(),
                data_type,
            };

            match c {
                Col::Input(_) => Col::Input(output),
                Col::Output(_) => Col::Output(output),
            }
        })
        .collect()
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
        sqlparser::ast::SelectItem::QualifiedWildcard(_, _) => todo!(),
        sqlparser::ast::SelectItem::Wildcard(_) => todo!(),
    }
}

fn column_from_expr(expr: &sqlparser::ast::Expr) -> Column {
    match expr {
        sqlparser::ast::Expr::Identifier(ident) => Column {
            name: ident.value.clone(),
            ..Default::default()
        },
        sqlparser::ast::Expr::CompoundIdentifier(_) => todo!(),
        sqlparser::ast::Expr::Wildcard => todo!(),
        sqlparser::ast::Expr::QualifiedWildcard(_) => todo!(),
        sqlparser::ast::Expr::BinaryOp { left, .. } => column_from_expr(left),
        _ => todo!(),
    }
}

#[derive(Debug)]
enum Col {
    Input(Column),
    Output(Column),
}
// find columns as params (anything that's a BinaryOp?)
// find columns in select part

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
        sqlparser::ast::SetExpr::Update(_) => todo!(),
        sqlparser::ast::SetExpr::Table(_) => todo!(),
    }
}

fn columns(ast: &Vec<Statement>) -> Vec<Col> {
    ast.iter()
        .flat_map(|statement| match statement {
            Statement::Insert { columns, .. } => columns_from_idents(columns)
                .into_iter()
                .map(|c| Col::Input(c))
                .collect(),
            Statement::Query(query) => columns_from_query(&query),
            _ => vec![],
        })
        .collect::<Vec<_>>()
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
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
    let data_type = if is_null(&value.options) {
        DataType::Null(data_type.into())
    } else {
        data_type
    };

    Column { name, data_type }
}

fn is_null(value: &Vec<sqlparser::ast::ColumnOptionDef>) -> bool {
    value
        .iter()
        .find(|opt| opt.option == sqlparser::ast::ColumnOption::NotNull)
        .is_none()
}

fn table(ast: &Vec<Statement>) -> Table {
    let columns = ast
        .iter()
        .flat_map(|statement| match statement {
            Statement::CreateTable { columns, .. } => {
                columns.iter().map(column).collect::<Vec<_>>()
            }
            _ => todo!(),
        })
        .collect();
    // let name: String = ast
    //     .iter()
    //     .map(|statement| match statement {
    //         Statement::CreateTable { name, .. } => name
    //             .0
    //             .last()
    //             .expect(&format!(
    //                 "could not find table name in create table statement: {}",
    //                 statement
    //             ))
    //             .to_string(),
    //         _ => todo!(),
    //     })
    //     .collect::<Vec<String>>()
    //     .last()
    //     .expect("could not find table name in create table statement")
    //     .clone();

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

fn is_execute(ast: &Vec<Statement>) -> bool {
    ast.iter()
        .find(|statement| match statement {
            // TODO: check returning and return false here instead?
            Statement::Insert { .. } | Statement::Update { .. } | Statement::Delete { .. } => true,
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
