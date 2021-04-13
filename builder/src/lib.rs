use quote::{quote, quote_spanned};
use proc_macro::{TokenStream};
use syn::{parse_macro_input, parse_quote, DeriveInput,  NestedMeta, Lit};
use proc_macro_roids::{DeriveInputStructExt, FieldExt, DeriveInputExt};
use syn::spanned::Spanned;



#[proc_macro_derive(build,attributes(cmd))]
pub fn make(input :TokenStream)->TokenStream{
    let derive_input = parse_macro_input!(input as DeriveInput);
    impl_default(&derive_input)
}

fn  impl_default(derive_input: &syn::DeriveInput) ->TokenStream{
    let name = &derive_input.ident;
    let typeid={
        match derive_input.tag_parameter(&parse_quote!(cmd), &parse_quote!(typeid)).unwrap(){
            NestedMeta::Lit(value)=>{
                match value {
                    Lit::Int(v)=>{
                        v.to_string().parse::<u16>().expect("typeid type error not u16")
                    },
                    _=>panic!("typeid type error not u16")
                }
            },
            _=>panic!("typeid error")
        }
    };

    let write= derive_input.fields().iter().filter(|f|{!f.is_phantom_data()}).map(|f|{
        let name=&f.ident;
        quote_spanned!{
                   f.span()=>  om.write_(data,&self.#name);
                }
    });

    let read= derive_input.fields().iter().filter(|f|{!f.is_phantom_data()}).map(|f|{
        let name=&f.ident;
        quote_spanned!{
                   f.span()=>  om.read_(data, &mut self.#name)?;
                }
    });

    let defs=  derive_input.fields().iter().filter(|f|{!f.is_phantom_data()}).map(|f|{
        let name=&f.ident;
        if let Some(x)= f.tag_parameter(&parse_quote!(cmd),&parse_quote!(default))
        {
            if let NestedMeta::Lit(value)=x{
                match value{
                    Lit::Int(v)=>{
                        quote_spanned! {
                           f.span()=>  #name: #v,
                        }
                    }
                    Lit::Float(v)=>{
                        quote_spanned! {
                           f.span()=>  #name: #v,
                        }
                    }
                    Lit::Bool(v)=>{
                        quote_spanned! {
                           f.span()=>  #name: #v,
                        }
                    }
                    Lit::Char(v)=>{
                        quote_spanned! {
                           f.span()=>  #name: #v,
                        }
                    }
                    Lit::Str(v)=>{
                        quote_spanned! {
                           f.span()=>  #name: #v.to_string(),
                        }
                    }
                    _=>{
                        quote_spanned! {
                           f.span()=>  #name: ::core::default::Default::default(),
                        }
                    }
                }
            }
            else{
                quote_spanned! {
                   f.span()=>  #name: ::core::default::Default::default(),
                }
            }
        }
        else {
            quote_spanned! {
                   f.span()=>  #name: ::core::default::Default::default(),
            }
        }
    });

    let expanded = quote! {
            impl xxlib::ISerdeTypeId for #name{
                #[inline]
                fn type_id() -> u16 where Self: Sized {
                    #typeid
                }
            }
            impl xxlib::ISerde for #name {
                #[inline(always)]
                fn get_offset_addr(&self) -> *mut u32 {
                    &self.__offset as * const u32 as *mut u32
                }
                #[inline(always)]
                fn write_to(&self,om: &xxlib::ObjectManager,data: &mut xxlib::Data) {
                    #( #write)*
                }
                #[inline(always)]
                fn read_from(&mut self,om: &xxlib::ObjectManager, data:&mut xxlib::DataReader) -> anyhow::Result<()> {
                    #( #read)*
                    Ok(())
                }
                #[inline(always)]
                fn get_type_id(&self) -> u16 {
                   use  xxlib::ISerdeTypeId;
                   Self::type_id()
                }
            }


            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::default::Default for #name {
                #[inline]
                fn default() -> #name {
                    #name {
                          #( #defs)*
                    }
                }
            }

            impl ToString for #name{
                #[inline]
                fn to_string(&self) -> String {
                     format!("{:?}",self)
                }
            }
    };

    return TokenStream::from(expanded);
}

