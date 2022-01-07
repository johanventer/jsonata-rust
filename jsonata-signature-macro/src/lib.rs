use jsonata_signatures::{parse, Arg, ArgKind, Flags};
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn signature(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = syn::parse_macro_input!(attr as syn::AttributeArgs);
    let item = syn::parse_macro_input!(item as syn::ItemFn);

    if attr.is_empty() {
        panic!("Must provide a signature");
    }

    let sig_args = match &attr[0] {
        syn::NestedMeta::Lit(syn::Lit::Str(sig)) => parse(&sig.value())
            .unwrap()
            .iter()
            .map(arg_to_expr)
            .collect::<Vec<syn::Expr>>(),
        _ => panic!("Signature must be a literal string"),
    };

    let sig_args_len = sig_args.len();
    let fn_name = item.sig.ident.to_string();
    let fn_vis = item.vis.clone();
    let sig_name = quote::format_ident!("__{}__signature", fn_name);

    let result = quote::quote! {
        lazy_static::lazy_static! {
            #fn_vis static ref #sig_name: [jsonata_signatures::Arg; #sig_args_len] = [ #(#sig_args, )* ];
        }
        #item
    };

    TokenStream::from(result)
}

fn arg_to_expr(arg: &Arg) -> syn::Expr {
    let arg = arg_to_str(arg);
    let arg = arg.parse().unwrap();
    let arg: syn::Expr = syn::parse(arg).unwrap();
    arg
}

fn arg_to_str(arg: &Arg) -> String {
    let kind = kind_to_str(&arg.kind);
    let flags = flags_to_str(&arg.flags);
    let arg = format!(
        r#"
            jsonata_signatures::Arg {{
                kind: {},
                flags: {}
            }}
        "#,
        kind, flags,
    );
    arg
}

fn kind_to_str(kind: &ArgKind) -> String {
    match kind {
        ArgKind::Null => "jsonata_signatures::ArgKind::Null".to_string(),
        ArgKind::Bool => "jsonata_signatures::ArgKind::Bool".to_string(),
        ArgKind::Number => "jsonata_signatures::ArgKind::Number".to_string(),
        ArgKind::String => "jsonata_signatures::ArgKind::String".to_string(),
        ArgKind::Object => "jsonata_signatures::ArgKind::Object".to_string(),
        ArgKind::Array(None) => "jsonata_signatures::ArgKind::Array(None)".to_string(),
        ArgKind::Array(Some(array_kind)) => format!(
            "jsonata_signatures::ArgKind::Array(Some(Box::new({})))",
            kind_to_str(array_kind)
        ),
        ArgKind::Function(None) => "jsonata_signatures::ArgKind::Function(None)".to_string(),
        ArgKind::Function(Some(args)) => format!(
            "jsonata_signatures::ArgKind::Function(Some(vec![{}]))",
            args.iter()
                .map(arg_to_str)
                .collect::<Vec<String>>()
                .join(", ")
        ),
        ArgKind::Or(args) => format!(
            "jsonata_signatures::ArgKind::Or(vec![{}])",
            args.iter()
                .map(kind_to_str)
                .collect::<Vec<String>>()
                .join(", ")
        ),
    }
}

fn flags_to_str(flags: &Flags) -> String {
    if *flags == Flags::empty() {
        return "jsonata_signatures::Flags::empty()".to_string();
    }
    let mut flag_str = Vec::new();
    if flags.contains(Flags::ONE_OR_MORE) {
        flag_str.push("jsonata_signatures::Flags::ONE_OR_MORE.bits()");
    }
    if flags.contains(Flags::OPTIONAL) {
        flag_str.push("jsonata_signatures::Flags::OPTIONAL.bits()");
    }
    if flags.contains(Flags::ACCEPT_CONTEXT) {
        flag_str.push("jsonata_signatures::Flags::ACCEPT_CONTEXT.bits()");
    }
    format!(
        "jsonata_signatures::Flags::from_bits_truncate({})",
        flag_str.join(" | ")
    )
}
