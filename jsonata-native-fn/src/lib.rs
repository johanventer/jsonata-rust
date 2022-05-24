use jsonata_signatures::{parse as parse_signature, Signature};
use proc_macro2::TokenStream;
use proc_macro_error::{abort, abort_call_site, proc_macro_error};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Item, ItemFn, Lit, Result};

#[proc_macro_attribute]
#[proc_macro_error]
pub fn jsonata(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let (ast, options) = parse(attr.into(), item.into());

    let output = quote!(#ast);

    output.into()
}

#[derive(Debug)]
struct Options {
    signature: Option<Signature>,
}

impl Parse for Options {
    fn parse(input: ParseStream) -> Result<Self> {
        const SIG_ERROR: &str = "the first argument should be a signature string";
        const SIG_HELP: &str =
            "The first argument should be a signature string, e.g. `#[jsonata(\"<b>\")]`";

        if input.is_empty() {
            Ok(Self { signature: None })
        } else {
            let signature = match input.parse::<Lit>() {
                Ok(Lit::Str(signature)) => match parse_signature(&signature.value()) {
                    Ok(signature) => signature,
                    Err(e) => {
                        abort!(signature, e; help = "Signature parsing failed, check the signature syntax")
                    }
                },
                Ok(l) => abort!(l, SIG_ERROR; help = SIG_HELP),
                Err(e) => abort!(e.span(), SIG_ERROR; help = SIG_HELP),
            };

            if !input.is_empty() {
                abort_call_site!("trailing characters in macro arguments")
            }

            Ok(Self {
                signature: Some(signature),
            })
        }
    }
}

fn parse(attr: TokenStream, item: TokenStream) -> (ItemFn, Options) {
    let ast = match syn::parse2::<Item>(item) {
        Ok(Item::Fn(item)) => item,
        Ok(item) => abort!(
            item,
            "item is not a function";
            help = "`#[jsonata]` can only be used on functions"
        ),
        _ => unreachable!(),
    };

    let options = syn::parse2::<Options>(attr).unwrap();

    (ast, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn it_works() {
        parse(
            quote!(),
            quote!(
                fn test() {}
            ),
        );
    }
}
