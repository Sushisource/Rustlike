extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{Data, Field, Fields, Ident, Path, PathSegment, Type, TypePath};

#[proc_macro_derive(CenterOriginRect)]
pub fn cor_derive(input: TokenStream) -> TokenStream {
  let ast = syn::parse(input).unwrap();
  // Build the impl
  let gen = impl_cor(&ast);
  // Return the generated impl
  gen.into()
}

fn impl_cor(ast: &syn::DeriveInput) -> quote::Tokens {
  let type_name = Ident::from("CenteredRect");
  let type_pathseg: PathSegment = type_name.into();
  let type_path: Path = type_pathseg.into();
  let name = &ast.ident;
  // Find the field of the struct which is the CenteredRect
  if let Data::Struct(ref sstruct) = ast.data {
    if let Fields::Named(ref fields) = sstruct.fields {
      let maybe_crfield = fields
        .named
        .iter()
        .find(|f| f.ty == Type::Path(TypePath { qself: None, path: type_path.clone() }));
      if let Some(&Field { ident: Some(identity), .. }) = maybe_crfield {
        quote! {
          impl CenterOriginRect for #name {
            fn center(&self) -> Point { self.#identity.center }
            fn width(&self) -> f32 { self.#identity.width }
            fn height(&self) -> f32 { self.#identity.height }
          }
        }
      } else {
        panic!("No field with type CenteredRect found")
      }
    } else {
      panic!("No named fields (must declare a CenteredRect field)!")
    }
  } else {
    panic!("Must be used on struct")
  }
}
