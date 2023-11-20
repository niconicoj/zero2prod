use proc_macro2::{TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Attribute, ReturnType, Signature, Visibility,
};

pub(crate) fn integration_test(item: TokenStream) -> TokenStream {
    let input: ItemFn = match syn::parse2(item.clone()) {
        Ok(input) => input,
        Err(e) => return token_stream_with_error(item, e),
    };

    let tokens = parse_knobs(input);
    tokens
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
    outer_attrs: Vec<Attribute>,
    vis: Visibility,
    sig: Signature,
    brace_token: syn::token::Brace,
    inner_attrs: Vec<Attribute>,
    stmts: Vec<proc_macro2::TokenStream>,
}

impl ItemFn {
    /// Get the body of the function item in a manner so that it can be
    /// conveniently used with the `quote!` macro.
    fn body(&self) -> Body<'_> {
        Body {
            brace_token: self.brace_token,
            stmts: &self.stmts,
        }
    }

    /// Convert our local function item into a token stream.
    fn into_tokens(
        self,
        header: proc_macro2::TokenStream,
        body: proc_macro2::TokenStream,
    ) -> TokenStream {
        let mut tokens = proc_macro2::TokenStream::new();
        header.to_tokens(&mut tokens);

        // Outer attributes are simply streamed as-is.
        for attr in self.outer_attrs {
            attr.to_tokens(&mut tokens);
        }

        // Inner attributes require extra care, since they're not supported on
        // blocks (which is what we're expanded into) we instead lift them
        // outside of the function. This matches the behaviour of `syn`.
        for mut attr in self.inner_attrs {
            attr.style = syn::AttrStyle::Outer;
            attr.to_tokens(&mut tokens);
        }

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
        // This parse implementation has been largely lifted from `syn`, with
        // the exception of:
        // * We don't have access to the plumbing necessary to parse inner
        //   attributes in-place.
        // * We do our own statements parsing to avoid recursively parsing
        //   entire statements and only look for the parts we're interested in.

        let outer_attrs = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        let sig: Signature = input.parse()?;

        let content;
        let brace_token = braced!(content in input);
        let inner_attrs = Attribute::parse_inner(&content)?;

        let mut buf = proc_macro2::TokenStream::new();
        let mut stmts = Vec::new();

        while !content.is_empty() {
            if let Some(semi) = content.parse::<Option<syn::Token![;]>>()? {
                semi.to_tokens(&mut buf);
                stmts.push(buf);
                buf = proc_macro2::TokenStream::new();
                continue;
            }

            // Parse a single token tree and extend our current buffer with it.
            // This avoids parsing the entire content of the sub-tree.
            buf.extend([content.parse::<TokenTree>()?]);
        }

        if !buf.is_empty() {
            stmts.push(buf);
        }

        Ok(Self {
            outer_attrs,
            vis,
            sig,
            brace_token,
            inner_attrs,
            stmts,
        })
    }
}

struct Body<'a> {
    brace_token: syn::token::Brace,
    // Statements, with terminating `;`.
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
