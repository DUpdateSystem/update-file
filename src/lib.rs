mod utils;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Error, LitStr};
use utils::get_content;

#[proc_macro]
pub fn get_content_const(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GetContentInput);
    let prefix_src = "src/";
    let file_path = format!("{}{}", prefix_src, input.file_path.value());
    let content = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(err) => {
            let error_token_stream =
                Error::new_spanned(&input.file_path, format!("Error reading file: {}", err))
                    .to_compile_error();
            return TokenStream::from(error_token_stream);
        }
    };
    let start = input.start.value();
    let end = input.end.as_ref().map(|s| s.value());

    match get_content(&content, &start, end.as_deref()) {
        Ok(content) => {
            let expanded = quote! { #content };
            TokenStream::from(expanded)
        }
        Err(err) => {
            let error_token_stream = Error::new_spanned(&input.file_path, err).to_compile_error();
            TokenStream::from(error_token_stream)
        }
    }
}

struct GetContentInput {
    file_path: LitStr,
    start: LitStr,
    end: Option<LitStr>,
}

impl syn::parse::Parse for GetContentInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let file_path = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let start = input.parse()?;
        let end = if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(GetContentInput {
            file_path,
            start,
            end,
        })
    }
}
