use crate::context::{Ctxt, Opts};
use crate::symbol;

/// Parse attributes.
pub(crate) fn parse(cx: &Ctxt<'_>) -> Result<Opts, ()> {
    let mut opts = Opts::default();

    for attr in &cx.ast.attrs {
        if attr.path() != symbol::KEY {
            continue;
        }

        let result = attr.parse_nested_meta(|input| {
            if input.path == symbol::BITSET {
                opts.bitset = Some(input.input.span());
            } else {
                return Err(syn::Error::new(input.input.span(), "Unsupported attribute"));
            }

            Ok(())
        });

        if let Err(error) = result {
            cx.error(error);
        }
    }

    Ok(opts)
}
