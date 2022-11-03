use syn::spanned::Spanned;
use syn::{Meta, NestedMeta};

use crate::context::{Ctxt, Opts};
use crate::symbol;

/// Parse attributes.
pub(crate) fn parse(cx: &Ctxt<'_>) -> Result<Opts, ()> {
    let mut opts = Opts::default();

    for attr in &cx.ast.attrs {
        if attr.path != symbol::KEY {
            continue;
        }

        let meta = cx.fallible(|| attr.parse_meta())?;

        let nested = match meta {
            Meta::List(meta) => meta.nested.into_iter(),
            other => {
                cx.error(other.span(), "unsupported attribute");
                return Err(());
            }
        };

        for meta in nested {
            match meta {
                NestedMeta::Meta(Meta::Path(p)) if p == symbol::BITSET => {
                    opts.bitset = Some(p.span());
                }
                other => {
                    cx.error(other.span(), "unsupported attribute");
                    return Err(());
                }
            }
        }
    }

    Ok(opts)
}
