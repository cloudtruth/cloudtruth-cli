mod use_harness;
use syn::ItemFn;
use use_harness::gen_use_harness;

#[proc_macro_attribute]
pub fn use_harness(
    _: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let test_fn = syn::parse::<ItemFn>(item).expect("Could not parse integration_test function");
    gen_use_harness(test_fn).into()
}
