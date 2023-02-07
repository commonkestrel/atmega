// Partialy adapted from https://github.com/rust-embedded/cortex-m-rt/blob/master/macros/src/lib.rs

use proc_macro::TokenStream;
use proc_macro2::{Span, Ident};
use syn::{LitStr, Visibility, VisPublic, spanned::Spanned, token::Pub};

#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
enum Interrupt {
    RESET        = 0,
    INT0         = 1,
    INT1         = 2,
    PCINT0       = 3,
    PCINT1       = 4,
    PCINT2       = 5,
    WDT          = 6,
    TIMER2_COMPA = 7,
    TIMER2_COMPB = 8,
    TIMER2_OVF   = 9,
    TIMER1_CAPT  = 10,
    TIMER1_COMPA = 11,
    TIMER1_COMPB = 12,
    TIMER1_OVF   = 13,
    TIMER0_COMPA = 14,
    TIMER0_COMPB = 15,
    TIMER0_OVF   = 16,
    SPI_STC      = 17,
    USART_RX     = 18,
    USART_UDRE   = 19,
    USART_TX     = 20,
    ADC          = 21,
    EE_READY     = 22,
    ANALOG_COMP  = 23,
    TWI          = 24,
    SPM_READY    = 25,
}

impl Interrupt {
    fn ident(value: &Ident) -> Option<Self> {
        let ident: &str = &value.to_string();

        use Interrupt::*;
        match ident {
            "RESET"        => Some(RESET),
            "INT0"         => Some(INT0),
            "INT1"         => Some(INT1),
            "PCINT0"       => Some(PCINT0),
            "PCINT1"       => Some(PCINT1),
            "PCINT2"       => Some(PCINT2),
            "WDT"          => Some(WDT),
            "TIMER2_COMPA" => Some(TIMER2_COMPA),
            "TIMER2_COMPB" => Some(TIMER2_COMPB),
            "TIMER2_OVF"   => Some(TIMER2_OVF),
            "TIMER1_CAPT"  => Some(TIMER1_CAPT),
            "TIMER1_COMPA" => Some(TIMER1_COMPA),
            "TIMER1_COMPB" => Some(TIMER1_COMPB),
            "TIMER1_OVF"   => Some(TIMER1_OVF),
            "TIMER0_COMPA" => Some(TIMER0_COMPA),
            "TIMER0_COMPB" => Some(TIMER0_COMPB),
            "TIMER0_OVF"   => Some(TIMER0_OVF),
            "SPI_STC"      => Some(SPI_STC),
            "USART_RX"     => Some(USART_RX),
            "USART_UDRE"   => Some(USART_UDRE),
            "USART_TX"     => Some(USART_TX),
            "ADC"          => Some(ADC),
            "EE_READY"     => Some(EE_READY),
            "ANALOG_COMP"  => Some(ANALOG_COMP),
            "TWI"          => Some(TWI),
            "SPM_READY"    => Some(SPM_READY),
            _ => None
        }
    }
    
    fn vector(&self) -> String {
        format!("__vector_{}", *self as u8)
    }
}

/// Exports the function as the matching interrupt handler
/// 
/// # Requirements
/// Requires the 'abi_avr_interrupt" feature, which can be enabled by adding #![feature(abi_avr_interrupt)] to the top of the file
#[proc_macro_attribute]
pub fn interrupt(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut f: syn::ItemFn = syn::parse(item).expect("'#[interrupt]' must be called on a function");
    let fnspan = f.span();

    if !attr.is_empty() {
        return syn::parse::Error::new(Span::call_site(), "This macro accepts no arguments")
            .to_compile_error()
            .into()
    }

    let valid = f.sig.constness.is_none()
        && f.sig.unsafety.is_some()
        && matches!(f.vis, syn::Visibility::Inherited)
        && f.sig.abi.is_none()
        && f.sig.inputs.is_empty()
        && f.sig.generics.params.is_empty()
        && f.sig.generics.where_clause.is_none()
        && match f.sig.output {
            syn::ReturnType::Default => true,
            syn::ReturnType::Type(_, ref ty) => match **ty {
                syn::Type::Tuple(ref tuple) => tuple.elems.is_empty(),
                syn::Type::Never(_) => true,
                _ => false,
            },
        };

    if !valid {
        return syn::parse::Error::new(fnspan, "#[interrupt] handlers must have the function signature unsafe fn() [-> (),-> !]")
            .to_compile_error()
            .into()
    }

    let interrupt = match Interrupt::ident(&f.sig.ident) {
        Some(i) => i,
        None => {
            return syn::parse::Error::new(fnspan, "Interrupt name not recognized. See atmega::interrupt::Interrupt for all options")
                .to_compile_error()
                .into()
        }
    };
    let vector = interrupt.vector();

    // No idea how this works, just let auto-complete fill in the gaps
    // Adds 'pub' to make public and 'extern "avr-interrupt" for the linker
    f.vis = Visibility::Public(VisPublic { pub_token: Pub(fnspan) });
    f.sig.abi = Some(syn::Abi { name: Some(LitStr::new("avr-interrupt", fnspan)), extern_token: syn::token::Extern { span: fnspan } });

    quote::quote!(
        #[doc(hidden)]
        #[allow(non_snake_case)]
        #[export_name = #vector]
        #f
    ).into()
}
