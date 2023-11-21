use proc_macro2::{TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    ReturnType, Signature, Visibility,
};

pub(crate) fn integration_test(item: TokenStream) -> TokenStream {
    let input: ItemFn = match syn::parse2(item.clone()) {
        Ok(input) => input,
        Err(e) => return token_stream_with_error(item, e),
    };

    parse_knobs(input)
}

fn parse_knobs(mut input: ItemFn) -> TokenStream {
    input.sig.asyncness = None;
    input.sig.inputs.clear();
    input.sig.output = ReturnType::Default;

    let header = quote! {
        #[::core::prelude::v1::test]
    };

    let body = input.body();
    let body = quote! {
        zero2prod_core::testing::run_test(|test_app: zero2prod_core::testing::TestApp| { Box::pin(async move #body)});
    };

    input.into_tokens(header, body)
}

#[derive(Debug)]
struct ItemFn {
    vis: Visibility,
    sig: Signature,
    brace_token: syn::token::Brace,
    stmts: Vec<proc_macro2::TokenStream>,
}

impl ItemFn {
    fn body(&self) -> Body<'_> {
        Body {
            brace_token: self.brace_token,
            stmts: &self.stmts,
        }
    }

    fn into_tokens(
        self,
        header: proc_macro2::TokenStream,
        body: proc_macro2::TokenStream,
    ) -> TokenStream {
        let mut tokens = proc_macro2::TokenStream::new();
        header.to_tokens(&mut tokens);

        self.vis.to_tokens(&mut tokens);
        self.sig.to_tokens(&mut tokens);

        self.brace_token.surround(&mut tokens, |tokens| {
            body.to_tokens(tokens);
        });

        tokens
    }
}

impl Parse for ItemFn {
    #[inline]
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let vis: Visibility = input.parse()?;
        let sig: Signature = input.parse()?;

        let content;
        let brace_token = braced!(content in input);

        let mut buf = proc_macro2::TokenStream::new();
        let mut stmts = Vec::new();

        while !content.is_empty() {
            if let Some(semi) = content.parse::<Option<syn::Token![;]>>()? {
                semi.to_tokens(&mut buf);
                stmts.push(buf);
                buf = proc_macro2::TokenStream::new();
                continue;
            }

            buf.extend([content.parse::<TokenTree>()?]);
        }

        if !buf.is_empty() {
            stmts.push(buf);
        }

        Ok(Self {
            vis,
            sig,
            brace_token,
            stmts,
        })
    }
}

struct Body<'a> {
    brace_token: syn::token::Brace,
    stmts: &'a [TokenStream],
}

impl ToTokens for Body<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.brace_token.surround(tokens, |tokens| {
            for stmt in self.stmts {
                stmt.to_tokens(tokens);
            }
        });
    }
}

fn token_stream_with_error(mut tokens: TokenStream, error: syn::Error) -> TokenStream {
    tokens.extend(error.into_compile_error());
    tokens
}
