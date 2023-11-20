use proc_macro::TokenStream;

mod entry;

#[proc_macro_attribute]
pub fn integration_test(_args: TokenStream, item: TokenStream) -> TokenStream {
    entry::integration_test(item.into()).into()
}
