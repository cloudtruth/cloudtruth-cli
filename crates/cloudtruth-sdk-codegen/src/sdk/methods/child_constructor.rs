use std::rc::Rc;

use proc_macro2::Ident;
use syn::{parse_quote, Field, FnArg, Type};

use crate::ident;
use crate::sdk::SdkObject;

use super::SdkMethod;

#[derive(Clone)]
pub struct SdkChildConstructor {
    fn_name: Rc<Ident>,
    child_type_name: Rc<Ident>,
    parent_fields: Vec<Ident>,
    args: Vec<(Ident, Type)>,
}

impl SdkChildConstructor {
    pub fn new(parent: &SdkObject, child: &SdkObject) -> Self {
        Self {
            fn_name: child.name().clone(),
            child_type_name: child.type_name().clone(),
            parent_fields: parent
                .fields()
                .iter()
                .map(|Field { ident, .. }| ident.clone().unwrap())
                .collect(),
            args: Vec::new(),
        }
    }

    pub fn add_arg(&mut self, arg_name: &str, arg_type: Type) {
        let arg_name = ident!(arg_name);
        self.args.push((arg_name, arg_type));
    }
}

impl SdkMethod for SdkChildConstructor {
    fn generate_fn(&self) -> syn::ItemFn {
        let Self {
            fn_name,
            child_type_name,
            parent_fields,
            args,
        } = self;
        let arg_names = args.iter().map(|(name, _)| name);
        let args = args
            .iter()
            .map::<FnArg, _>(|(name, ty)| parse_quote!( #name : #ty ));
        parse_quote! {
            pub fn #fn_name(&self, #(#args,)*) -> #child_type_name {
                let Self { #(#parent_fields,)* } = self;
                #child_type_name { #(#parent_fields: #parent_fields.clone(),)* #(#arg_names: #arg_names.into(),)* }
            }
        }
    }
}
