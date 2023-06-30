use quote::quote;
use syn::ItemFn;

pub(crate) fn gen_use_harness(test_fn: ItemFn) -> proc_macro2::TokenStream {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = test_fn;
    quote! {
        #(#attrs)*
        #vis #sig
        {
            let _orig_hook = std::panic::take_hook();
            cloudtruth_test_harness::install();
            let ret = {
                #block
            };
            std::panic::set_hook(_orig_hook);
            ret
        }
    }
}
