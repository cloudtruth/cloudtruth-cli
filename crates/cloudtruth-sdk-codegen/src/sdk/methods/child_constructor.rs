use proc_macro2::{Ident, Span};
use syn::{parse_quote, Field, FnArg, Type};

use crate::sdk::SdkObject;

use super::SdkMethod;

#[derive(Clone)]
pub struct SdkChildConstructor {
    name: Ident,
    child_name: Ident,
    parent_fields: Vec<Ident>,
    args: Vec<(Ident, Type)>,
}

impl SdkChildConstructor {
    pub fn new(parent: &SdkObject, child: &SdkObject) -> Self {
        Self {
            name: child.name().clone(),
            child_name: child.name().clone(),
            parent_fields: parent
                .fields()
                .iter()
                .map(|Field { ident, .. }| ident.clone().unwrap())
                .collect(),
            args: Vec::new(),
        }
    }

    pub fn add_arg(&mut self, arg_name: &str, arg_type: Type) {
        let arg_name = Ident::new(arg_name, Span::call_site());
        self.args.push((arg_name, arg_type));
    }
}

impl SdkMethod for SdkChildConstructor {
    fn generate_fn(&self) -> syn::ItemFn {
        let Self {
            name,
            child_name,
            parent_fields,
            args,
        } = self;
        let arg_names = args.iter().map(|(name, _)| name);
        let args = args
            .iter()
            .map::<FnArg, _>(|(name, ty)| parse_quote!( #name : #ty ));
        parse_quote! {
            pub fn #name(&self, #(#args,)*) -> #child_name {
                let Self { #(#parent_fields,)* } = self;
                #child_name { #(#parent_fields,)* #(#arg_names,)* }
            }
        }
    }
}
