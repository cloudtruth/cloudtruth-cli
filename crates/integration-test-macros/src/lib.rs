use quote::quote;
use syn::ItemFn;

#[proc_macro_attribute]
pub fn integration_test(
    _: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let test_fn = syn::parse::<ItemFn>(item).expect("Could not parse integration_test function");
    gen_integration_test(test_fn).into()
}

fn gen_integration_test(test_fn: ItemFn) -> proc_macro2::TokenStream {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = test_fn;
    quote! {
        #[test]
        #(#attrs)*
        #vis #sig
        {
            integration_test_harness::install();
            #block
        }
    }
}
