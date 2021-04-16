use quote::{quote, quote_spanned,format_ident};
use proc_macro::{TokenStream};
use syn::{Data,parse_macro_input, parse_quote, DeriveInput, NestedMeta, Lit, Field};
use proc_macro_roids::{DeriveInputStructExt, FieldExt, DeriveInputExt};
use syn::spanned::Spanned;



#[proc_macro_derive(build,attributes(cmd))]
pub fn make(input :TokenStream)->TokenStream{
    let derive_input = parse_macro_input!(input as DeriveInput);
    impl_default(&derive_input)
}

fn impl_default(derive_input: &syn::DeriveInput) ->TokenStream{
    let name = &derive_input.ident;
    let typeid={
        match derive_input.tag_parameter(&parse_quote!(cmd), &parse_quote!(typeid)){
            Some(value)=> {
                match value {
                    NestedMeta::Lit(value) => {
                        match value {
                            Lit::Int(v) => {
                                Some(v.to_string().parse::<u16>().expect("typeid type error not u16"))
                            },
                            _ => panic!("typeid type error not u16")
                        }
                    },
                    _=>panic!("typeid type error")
                }
            },
            _=>None
        }
    };


    let is_compatible= match derive_input.tag_parameter(&parse_quote!(cmd), &parse_quote!(compatible)){
        Some(p)=>{
            match p{
                NestedMeta::Lit(value)=>{
                    match value {
                        Lit::Bool(v)=>{
                            v.value
                        },
                        _=>false
                    }
                },
                _=>false
            }
        },
        None=>false
    };

    let write= derive_input.fields().iter().filter(|f|{!f.is_phantom_data()}).map(|f|{
        let name=&f.ident;
        quote_spanned! {
            f.span()=>  om.write_(data,&self.#name)?;
        }

    });


    let defs=  derive_input.fields().iter().filter(|f|{!f.is_phantom_data()}).map(|f|{
        let name=&f.ident;
        if let Some(x)= f.tag_parameter(&parse_quote!(cmd),&parse_quote!(default))
        {
            let default=get_fmt_default(f, x);
            quote_spanned! {
                   f.span()=> #name: #default,
                }
        }
        else {
            quote_spanned! {
                   f.span()=>  #name: ::core::default::Default::default(),
            }
        }
    });


    return if !is_compatible {
        let read = derive_input.fields().iter().filter(|f| { !f.is_phantom_data() }).map(|f| {
            let name = &f.ident;
            quote_spanned! {
                   f.span()=>  om.read_(data, &mut self.#name)?;
            }
        });

        if let Some(typeid)=typeid {
            let expanded = quote! {
                impl xxlib::ISerdeTypeId for #name{
                    #[inline(always)]
                    fn type_id() -> u16 where Self: Sized {
                        #typeid
                    }
                    #[inline]
                    fn get_type_id(&self) -> u16 {
                       use xxlib::ISerdeTypeId;
                       Self::type_id()
                    }
                }
                impl xxlib::ISerde for #name {
                    #[inline]
                    fn write_to(&self,om: &xxlib::ObjectManager,data: &mut xxlib::Data)->anyhow::Result<()> {
                        #( #write)*
                        Ok(())
                    }
                    #[inline]
                    fn read_from(&mut self,om: &xxlib::ObjectManager, data:&mut xxlib::DataReader) -> anyhow::Result<()> {
                        #( #read)*
                        Ok(())
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

            TokenStream::from(expanded)
        }else {
            let expanded = quote! {
                impl xxlib::IStruct for #name {
                    #[inline]
                    fn write_to(&self,om: &xxlib::ObjectManager,data: &mut xxlib::Data)->anyhow::Result<()> {
                        #( #write)*
                        Ok(())
                    }
                    #[inline]
                    fn read_from(&mut self,om: &xxlib::ObjectManager, data:&mut xxlib::DataReader) -> anyhow::Result<()> {
                        #( #read)*
                        Ok(())
                    }
                }
                impl xxlib::IWriteInner for #name{
                    #[inline]
                    fn write_(&self, om: &ObjectManager, data: &mut Data) -> anyhow::Result<()> {
                        self.write_to(om,data)
                    }
                }
                impl xxlib::IReadInner for #name{
                    #[inline]
                    fn read_(&mut self, om: &ObjectManager, data: &mut DataReader) -> anyhow::Result<()> {
                        self.read_from(om,data)
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
            TokenStream::from(expanded)
        }
    }else{
        let read = derive_input.fields().iter().filter(|f| { !f.is_phantom_data() }).map(|f| {
            let name = &f.ident;
            let default= if let Some(x)= f.tag_parameter(&parse_quote!(cmd),&parse_quote!(default)){
                let set_default=get_fmt_default(f, x);
                quote_spanned! {
                   f.span()=>   self.#name= #set_default;
                }
            }else{
                quote_spanned! {
                   f.span()=>   self.#name=  ::core::default::Default::default();
                }
            };
            quote! {
                    if read.len()>0 {
                        om.read_(&mut read, &mut self.#name)?;
                    }else{
                        #default;
                    }
                }
        });
        if let Some(typeid)=typeid {
            let expanded = quote! {
                impl xxlib::ISerdeTypeId for #name{
                    #[inline(always)]
                    fn type_id() -> u16 where Self: Sized {
                        #typeid
                    }
                    #[inline]
                    fn get_type_id(&self) -> u16 {
                       use xxlib::ISerdeTypeId;
                       Self::type_id()
                    }
                }
                impl xxlib::ISerde for #name {
                    #[inline]
                    fn write_to(&self,om: &xxlib::ObjectManager,data: &mut xxlib::Data)->anyhow::Result<()> {
                        let bak=data.len();
                        data.write_fixed(&0u32);
                        #( #write)*
                        data.write_fixed_at(bak,(data.len()-bak) as u32)?;
                        Ok(())
                    }
                    #[inline]
                    fn read_from(&mut self,om: &xxlib::ObjectManager, data:&mut xxlib::DataReader) -> anyhow::Result<()> {
                        let end_offset = data.read_fixed::<u32>()? as usize - 4usize;
                        anyhow::ensure!(end_offset<=data.len(),"struct:'{}' read_from offset error end_offset:{} > have len:{}", core::any::type_name::<Self>(),end_offset,data.len());
                        let mut read = xxlib::DataReader::from(&data[..end_offset]);
                        #( #read)*
                        data.advance(end_offset)?;
                        Ok(())
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
            TokenStream::from(expanded)
        }else {
            let expanded = quote! {
                impl xxlib::IStruct for #name {
                    #[inline]
                    fn write_to(&self,om: &xxlib::ObjectManager,data: &mut xxlib::Data)->anyhow::Result<()> {
                        let bak=data.len();
                        data.write_fixed(&0u32);
                        #( #write)*
                        data.write_fixed_at(bak,(data.len()-bak) as u32)?;
                        Ok(())
                    }
                    #[inline]
                    fn read_from(&mut self,om: &xxlib::ObjectManager, data:&mut xxlib::DataReader) -> anyhow::Result<()> {
                        let end_offset = data.read_fixed::<u32>()? as usize - 4usize;
                        anyhow::ensure!(end_offset<=data.len(),"struct:'{}' read_from offset error end_offset:{} > have len:{}", core::any::type_name::<Self>(),end_offset,data.len());
                        let mut read = xxlib::DataReader::from(&data[..end_offset]);
                        #( #read)*
                        data.advance(end_offset)?;
                        Ok(())
                    }
                }
                impl xxlib::IWriteInner for #name{
                    #[inline]
                    fn write_(&self, om: &ObjectManager, data: &mut Data) -> anyhow::Result<()> {
                        self.write_to(om,data)
                    }
                }
                impl xxlib::IReadInner for #name{
                    #[inline]
                    fn read_(&mut self, om: &ObjectManager, data: &mut DataReader) -> anyhow::Result<()> {
                        self.read_from(om,data)
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
            TokenStream::from(expanded)
        }
    }
}

fn get_fmt_default(f: &Field,  x: NestedMeta) -> proc_macro2::TokenStream {
    return match x {
            NestedMeta::Lit(value) => {
                match value {
                    Lit::Int(v) => {
                        quote_spanned! {
                           f.span()=> #v
                        }
                    }
                    Lit::Float(v) => {
                        quote_spanned! {
                           f.span()=> #v
                        }
                    }
                    Lit::Bool(v) => {
                        quote_spanned! {
                           f.span()=> #v
                        }
                    }
                    Lit::Char(v) => {
                        quote_spanned! {
                           f.span()=> #v
                        }
                    }
                    Lit::Str(v) => {
                        quote_spanned! {
                           f.span()=> #v.to_string()
                        }
                    }
                    _ => {
                        quote_spanned! {
                           f.span()=> ::core::default::Default::default()
                        }
                    }
                }
            },
            NestedMeta::Meta(value) => {

                quote_spanned! {
                   f.span()=> #value
                }
            }
        }
}

#[proc_macro_attribute]
pub fn build_enum(args:TokenStream, input: TokenStream) -> TokenStream {
    let number_type =  format_ident!("{}",args.to_string().trim());
    let ast:syn::DeriveInput = syn::parse(input).unwrap();
    let name=&ast.ident;
    let expanded = if let Data::Enum(_) = ast.data {
        quote! {
             #[repr(#number_type)]
             #[derive(Copy, Clone,Debug,Eq, PartialEq)]
             #ast
             impl xxlib::manager::IWriteInner for #name{
                #[inline]
                fn write_(&self, om: &ObjectManager, data: &mut Data) -> anyhow::Result<()> {
                    let v:#number_type=unsafe{
                         std::mem::transmute(*self)
                    };
                    om.write_(data,&v)?;
                    Ok(())
                }
            }
            impl xxlib::manager::IReadInner for #name{
                #[inline]
                fn read_(&mut self, om: &ObjectManager, data: &mut DataReader) -> anyhow::Result<()> {
                    let mut v:#number_type = 0;
                    om.read_(data,&mut v)?;
                    unsafe{
                        *self=std::mem::transmute(v)
                    }
                    Ok(())
                }
            }
            impl Default for #name{
                #[inline]
                fn default() -> Self {
                    unsafe{
                        std::mem::transmute::<#number_type,#name>(0)
                    }
                }
            }
        }
    } else {
        quote! {
            #ast
        }
    };

    TokenStream::from(expanded)
}

