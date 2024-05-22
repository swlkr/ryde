use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Result};

pub fn derive_request_parts_macro(input: DeriveInput) -> Result<TokenStream> {
    let struct_ident = input.ident;

    let field_impls = match input.data {
        Data::Struct(DataStruct { fields, .. }) => fields
            .iter()
            .map(|field| {
                let ty = &field.ty;
                let ident = &field.ident;
                quote! {
                    #[async_trait]
                    impl<S> FromRequestParts<S> for #ty
                    where
                        #struct_ident: FromRef<S>,
                        S: Send + Sync,
                    {
                        type Rejection = Response;

                        async fn from_request_parts(
                            _parts: &mut axum::http::request::Parts,
                            state: &S,
                        ) -> std::result::Result<Self, Self::Rejection> {
                            let state = #struct_ident::from_ref(state);
                            Ok(state.#ident)
                        }
                    }
                }
            })
            .collect::<Vec<_>>(),
        Data::Enum(_) | Data::Union(_) => unimplemented!("Only structs are supported"),
    };

    Ok(quote! {
        #[async_trait]
        impl<S> FromRequestParts<S> for #struct_ident
        where
            #struct_ident: FromRef<S>,
            S: Send + Sync,
        {
            type Rejection = Response;

            async fn from_request_parts(
                _parts: &mut axum::http::request::Parts,
                state: &S,
            ) -> std::result::Result<Self, Self::Rejection> {
                let state = #struct_ident::from_ref(state);
                Ok(state)
            }
        }

        #(#field_impls)*
    })
}
