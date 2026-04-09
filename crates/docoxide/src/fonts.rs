use parley::FontContext;
use parley::fontique::{Collection, CollectionOptions, SourceCache};

pub fn build_font_context() -> FontContext {
    #[cfg(target_arch = "wasm32")]
    {
        use parley::fontique::{Blob, GenericFamily};
        use std::sync::Arc;
        const FONTS: &[&[u8]] = &[
            include_bytes!("../fonts/NotoSans-Variable.ttf"),
            include_bytes!("../fonts/NotoSans-Italic-Variable.ttf"),
        ];
        let mut collection = Collection::new(CollectionOptions {
            shared: false,
            system_fonts: false,
        });
        for font in FONTS {
            collection.register_fonts(Blob::new(Arc::new(font.to_vec()) as _), None);
        }
        if let Some(id) = collection.family_id("Noto Sans") {
            for generic in GenericFamily::all() {
                collection.set_generic_families(*generic, [id].into_iter());
            }
        }
        FontContext {
            source_cache: SourceCache::new_shared(),
            collection,
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        FontContext {
            source_cache: SourceCache::new_shared(),
            collection: Collection::new(CollectionOptions {
                shared: false,
                system_fonts: true,
            }),
        }
    }
}
