#![allow(unused_unsafe)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::redundant_static_lifetimes)]
#![allow(clippy::unused_unit)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::let_and_return)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::non_canonical_partial_ord_impl)]

use std::marker::{PhantomData, PhantomPinned};
use std::mem::ManuallyDrop;

use roc_std::roc_refcounted_noop_impl;
use roc_std::{RocBox, RocRefcounted};

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[repr(transparent)]
pub struct U2();

impl U2 {
    /// A tag named NoOp, which has no payload.
    pub const NoOp: Self = Self();

    /// Other `into_` methods return a payload, but since NoOp tag
    /// has no payload, this does nothing and is only here for completeness.
    pub fn into_NoOp(self) {
        ()
    }

    /// Other `as_` methods return a payload, but since NoOp tag
    /// has no payload, this does nothing and is only here for completeness.
    pub fn as_NoOp(&self) {
        ()
    }
}

impl core::fmt::Debug for U2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("U2::NoOp")
    }
}

roc_refcounted_noop_impl!(U2);

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[repr(u8)]
pub enum discriminant_U1 {
    Err = 0,
    Ok = 1,
}

impl core::fmt::Debug for discriminant_U1 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Err => f.write_str("discriminant_U1::Err"),
            Self::Ok => f.write_str("discriminant_U1::Ok"),
        }
    }
}

roc_refcounted_noop_impl!(discriminant_U1);

#[repr(C, align(4))]
pub union union_U1 {
    Err: U2,
    Ok: core::mem::ManuallyDrop<roc_std::RocStr>,
}

impl U1 {
    /// Returns which variant this tag union holds. Note that this never includes a payload!
    pub fn discriminant(&self) -> discriminant_U1 {
        unsafe {
            let bytes = core::mem::transmute::<&Self, &[u8; core::mem::size_of::<Self>()]>(self);

            core::mem::transmute::<u8, discriminant_U1>(*bytes.as_ptr().add(12))
        }
    }

    /// Internal helper
    fn set_discriminant(&mut self, discriminant: discriminant_U1) {
        let discriminant_ptr: *mut discriminant_U1 = (self as *mut U1).cast();

        unsafe {
            *(discriminant_ptr.add(12)) = discriminant;
        }
    }
}

#[repr(C)]
pub struct U1 {
    payload: union_U1,
    discriminant: discriminant_U1,
}

impl Clone for U1 {
    fn clone(&self) -> Self {
        use discriminant_U1::*;

        let payload = unsafe {
            match self.discriminant {
                Err => union_U1 {
                    Err: self.payload.Err.clone(),
                },
                Ok => union_U1 {
                    Ok: self.payload.Ok.clone(),
                },
            }
        };

        Self {
            discriminant: self.discriminant,
            payload,
        }
    }
}

impl core::fmt::Debug for U1 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use discriminant_U1::*;

        unsafe {
            match self.discriminant {
                Err => {
                    let field: &U2 = &self.payload.Err;
                    f.debug_tuple("U1::Err").field(field).finish()
                }
                Ok => {
                    let field: &roc_std::RocStr = &self.payload.Ok;
                    f.debug_tuple("U1::Ok").field(field).finish()
                }
            }
        }
    }
}

impl Eq for U1 {}

impl PartialEq for U1 {
    fn eq(&self, other: &Self) -> bool {
        use discriminant_U1::*;

        if self.discriminant != other.discriminant {
            return false;
        }

        unsafe {
            match self.discriminant {
                Err => self.payload.Err == other.payload.Err,
                Ok => self.payload.Ok == other.payload.Ok,
            }
        }
    }
}

impl Ord for U1 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for U1 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use discriminant_U1::*;

        use std::cmp::Ordering::*;

        match self.discriminant.cmp(&other.discriminant) {
            Less => Option::Some(Less),
            Greater => Option::Some(Greater),
            Equal => unsafe {
                match self.discriminant {
                    Err => self.payload.Err.partial_cmp(&other.payload.Err),
                    Ok => self.payload.Ok.partial_cmp(&other.payload.Ok),
                }
            },
        }
    }
}

impl core::hash::Hash for U1 {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        use discriminant_U1::*;

        unsafe {
            match self.discriminant {
                Err => self.payload.Err.hash(state),
                Ok => self.payload.Ok.hash(state),
            }
        }
    }
}

impl U1 {
    pub fn unwrap_Err(mut self) -> U2 {
        debug_assert_eq!(self.discriminant, discriminant_U1::Err);
        unsafe { self.payload.Err }
    }

    pub fn borrow_Err(&self) -> U2 {
        debug_assert_eq!(self.discriminant, discriminant_U1::Err);
        unsafe { self.payload.Err }
    }

    pub fn borrow_mut_Err(&mut self) -> &mut U2 {
        debug_assert_eq!(self.discriminant, discriminant_U1::Err);
        unsafe { &mut self.payload.Err }
    }

    pub fn is_Err(&self) -> bool {
        matches!(self.discriminant, discriminant_U1::Err)
    }

    pub fn unwrap_Ok(mut self) -> roc_std::RocStr {
        debug_assert_eq!(self.discriminant, discriminant_U1::Ok);
        unsafe { core::mem::ManuallyDrop::take(&mut self.payload.Ok) }
    }

    pub fn borrow_Ok(&self) -> &roc_std::RocStr {
        debug_assert_eq!(self.discriminant, discriminant_U1::Ok);
        use core::borrow::Borrow;
        unsafe { self.payload.Ok.borrow() }
    }

    pub fn borrow_mut_Ok(&mut self) -> &mut roc_std::RocStr {
        debug_assert_eq!(self.discriminant, discriminant_U1::Ok);
        use core::borrow::BorrowMut;
        unsafe { self.payload.Ok.borrow_mut() }
    }

    pub fn is_Ok(&self) -> bool {
        matches!(self.discriminant, discriminant_U1::Ok)
    }
}

impl U1 {
    pub fn Err(payload: U2) -> Self {
        Self {
            discriminant: discriminant_U1::Err,
            payload: union_U1 { Err: payload },
        }
    }

    pub fn Ok(payload: roc_std::RocStr) -> Self {
        Self {
            discriminant: discriminant_U1::Ok,
            payload: union_U1 {
                Ok: core::mem::ManuallyDrop::new(payload),
            },
        }
    }
}

impl Drop for U1 {
    fn drop(&mut self) {
        // Drop the payloads
        match self.discriminant() {
            discriminant_U1::Err => {}
            discriminant_U1::Ok => unsafe { core::mem::ManuallyDrop::drop(&mut self.payload.Ok) },
        }
    }
}

impl roc_std::RocRefcounted for U1 {
    fn inc(&mut self) {
        unimplemented!();
    }
    fn dec(&mut self) {
        unimplemented!();
    }
    fn is_refcounted() -> bool {
        true
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[repr(C)]
pub struct R1 {
    pub model: RocBox<()>,
    pub to_backend: U1,
}

impl roc_std::RocRefcounted for R1 {
    fn inc(&mut self) {
        self.to_backend.inc();
    }
    fn dec(&mut self) {
        self.to_backend.dec();
    }
    fn is_refcounted() -> bool {
        true
    }
}

#[derive(Clone, Default, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[repr(C)]
pub struct InternalAttr_Attribute {
    pub f0: roc_std::RocStr,
    pub f1: roc_std::RocStr,
}

impl roc_std::RocRefcounted for InternalAttr_Attribute {
    fn inc(&mut self) {
        self.f0.inc();
        self.f1.inc();
    }
    fn dec(&mut self) {
        self.f0.dec();
        self.f1.dec();
    }
    fn is_refcounted() -> bool {
        true
    }
}

#[derive(Clone, Default, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[repr(C)]
pub struct R3 {
    pub id: roc_std::RocStr,
    pub tagName: roc_std::RocStr,
    pub value: roc_std::RocStr,
    pub checked: bool,
}

impl roc_std::RocRefcounted for R3 {
    fn inc(&mut self) {
        self.id.inc();
        self.tagName.inc();
        self.value.inc();
    }
    fn dec(&mut self) {
        self.id.dec();
        self.tagName.dec();
        self.value.dec();
    }
    fn is_refcounted() -> bool {
        true
    }
}

#[derive(Clone, Default, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[repr(C)]
pub struct InternalEvent {
    pub button: i32,
    pub clientX: i32,
    pub clientY: i32,
    pub code: roc_std::RocStr,
    pub currentTarget: R3,
    pub eventType: roc_std::RocStr,
    pub key: roc_std::RocStr,
    pub target: R3,
    pub altKey: bool,
    pub ctrlKey: bool,
    pub metaKey: bool,
    pub preventDefault: bool,
    pub shiftKey: bool,
    pub stopPropagation: bool,
}

impl roc_std::RocRefcounted for InternalEvent {
    fn inc(&mut self) {
        self.code.inc();
        self.currentTarget.inc();
        self.eventType.inc();
        self.key.inc();
        self.target.inc();
    }
    fn dec(&mut self) {
        self.code.dec();
        self.currentTarget.dec();
        self.eventType.dec();
        self.key.dec();
        self.target.dec();
    }
    fn is_refcounted() -> bool {
        true
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct RocFunction_3569 {
    data: (),
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

impl RocFunction_3569 {
    // IMPORTANT: This originally moved self into the function but it was changed to take
    // a mutable pointer as I do not know the effect of move semantics for this type
    // whose size is unknown
    // This likely means using the pointer after it's been passed to this function is
    // undefined behaviour
    pub fn force_thunk(&mut self, arg0: InternalEvent) -> RocBox<()> {
        extern "C" {
            fn roc__frontend_view_for_host_0_caller(
                arg0: *const InternalEvent,
                closure_data: *mut u8,
                output: *mut RocBox<()>,
            );
        }

        let mut output = core::mem::MaybeUninit::uninit();

        unsafe {
            roc__frontend_view_for_host_0_caller(
                &arg0,
                self as *mut _ as *mut u8,
                output.as_mut_ptr(),
            );

            output.assume_init()
        }
    }
}
roc_refcounted_noop_impl!(RocFunction_3569);

#[derive(Debug)]
#[repr(C)]
pub struct InternalAttr_OnEvent {
    pub f0: roc_std::RocStr,
    pub f1: RocFunction_3569,
}

impl roc_std::RocRefcounted for InternalAttr_OnEvent {
    fn inc(&mut self) {
        self.f0.inc();
    }
    fn dec(&mut self) {
        self.f0.dec();
    }
    fn is_refcounted() -> bool {
        true
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[repr(u8)]
pub enum discriminant_InternalAttr {
    Alt = 0,
    Attribute = 1,
    Autocomplete = 2,
    Checked = 3,
    Class = 4,
    DataAttribute = 5,
    Disabled = 6,
    Hidden = 7,
    Href = 8,
    Id = 9,
    Multiple = 10,
    Name = 11,
    OnEvent = 12,
    Placeholder = 13,
    Readonly = 14,
    Required = 15,
    Selected = 16,
    Src = 17,
    Style = 18,
    Tabindex = 19,
    Title = 20,
    Type = 21,
    Value = 22,
}

impl core::fmt::Debug for discriminant_InternalAttr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Alt => f.write_str("discriminant_InternalAttr::Alt"),
            Self::Attribute => f.write_str("discriminant_InternalAttr::Attribute"),
            Self::Autocomplete => f.write_str("discriminant_InternalAttr::Autocomplete"),
            Self::Checked => f.write_str("discriminant_InternalAttr::Checked"),
            Self::Class => f.write_str("discriminant_InternalAttr::Class"),
            Self::DataAttribute => f.write_str("discriminant_InternalAttr::DataAttribute"),
            Self::Disabled => f.write_str("discriminant_InternalAttr::Disabled"),
            Self::Hidden => f.write_str("discriminant_InternalAttr::Hidden"),
            Self::Href => f.write_str("discriminant_InternalAttr::Href"),
            Self::Id => f.write_str("discriminant_InternalAttr::Id"),
            Self::Multiple => f.write_str("discriminant_InternalAttr::Multiple"),
            Self::Name => f.write_str("discriminant_InternalAttr::Name"),
            Self::OnEvent => f.write_str("discriminant_InternalAttr::OnEvent"),
            Self::Placeholder => f.write_str("discriminant_InternalAttr::Placeholder"),
            Self::Readonly => f.write_str("discriminant_InternalAttr::Readonly"),
            Self::Required => f.write_str("discriminant_InternalAttr::Required"),
            Self::Selected => f.write_str("discriminant_InternalAttr::Selected"),
            Self::Src => f.write_str("discriminant_InternalAttr::Src"),
            Self::Style => f.write_str("discriminant_InternalAttr::Style"),
            Self::Tabindex => f.write_str("discriminant_InternalAttr::Tabindex"),
            Self::Title => f.write_str("discriminant_InternalAttr::Title"),
            Self::Type => f.write_str("discriminant_InternalAttr::Type"),
            Self::Value => f.write_str("discriminant_InternalAttr::Value"),
        }
    }
}

roc_refcounted_noop_impl!(discriminant_InternalAttr);

#[repr(C, align(4))]
pub union union_InternalAttr {
    Alt: core::mem::ManuallyDrop<roc_std::RocStr>,
    Attribute: core::mem::ManuallyDrop<InternalAttr_Attribute>,
    Autocomplete: core::mem::ManuallyDrop<roc_std::RocStr>,
    Checked: bool,
    Class: core::mem::ManuallyDrop<roc_std::RocStr>,
    DataAttribute: core::mem::ManuallyDrop<InternalAttr_Attribute>,
    Disabled: bool,
    Hidden: bool,
    Href: core::mem::ManuallyDrop<roc_std::RocStr>,
    Id: core::mem::ManuallyDrop<roc_std::RocStr>,
    Multiple: bool,
    Name: core::mem::ManuallyDrop<roc_std::RocStr>,
    OnEvent: core::mem::ManuallyDrop<InternalAttr_OnEvent>,
    Placeholder: core::mem::ManuallyDrop<roc_std::RocStr>,
    Readonly: bool,
    Required: bool,
    Selected: bool,
    Src: core::mem::ManuallyDrop<roc_std::RocStr>,
    Style: core::mem::ManuallyDrop<roc_std::RocStr>,
    Tabindex: i32,
    Title: core::mem::ManuallyDrop<roc_std::RocStr>,
    Type: core::mem::ManuallyDrop<roc_std::RocStr>,
    Value: core::mem::ManuallyDrop<roc_std::RocStr>,
}

pub struct InternalAttr {
    data: (),
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

impl InternalAttr {
    pub fn size() -> u64 {
        extern "C" {
            #[link_name = "roc__frontend_view_for_host_0_size"]
            fn caller() -> u64;
        }

        let roc_function_size = unsafe { caller() };

        // --- Step 1: Determine the size of the `OnEvent` variant ---

        // Assume the function/closure object has the alignment of a pointer.
        let roc_function_align = align_of::<usize>();

        // Get the size and alignment of the first field in `InternalAttr_OnEvent`.
        let roc_str_size = size_of::<RocStr>();
        let roc_str_align = align_of::<RocStr>();

        // The offset of the second field (`f1`) is the size of the first (`f0`),
        // rounded up to a multiple of the second field's alignment.
        let function_field_offset =
            (roc_str_size + roc_function_align - 1) & !(roc_function_align - 1);

        // The unaligned size of the struct is the offset of the last field plus its size.
        let on_event_struct_unaligned_size = function_field_offset + roc_function_size;

        // The struct's total alignment is the max of its fields' alignments.
        let on_event_struct_align = max(roc_str_align, roc_function_align);

        // The final size of the `OnEvent` variant's data is its unaligned size
        // rounded up to a multiple of its own alignment.
        let size_of_on_event_variant = (on_event_struct_unaligned_size + on_event_struct_align - 1)
            & !(on_event_struct_align - 1);

        // --- Step 2: Find the maximum size among ALL union variants ---
        let mut max_variant_size = 0;
        max_variant_size = max(max_variant_size, size_of::<RocStr>());
        max_variant_size = max(max_variant_size, size_of::<InternalAttr_Attribute>());
        max_variant_size = max(max_variant_size, size_of::<bool>());
        max_variant_size = max(max_variant_size, size_of::<i32>());
        max_variant_size = max(max_variant_size, size_of_on_event_variant); // The one we just calculated

        // --- Step 3: Apply the union's overall alignment ---
        // The union is `#[repr(C, align(4))]`. Its final size must be the
        // smallest multiple of 4 that is >= max_variant_size.
        let union_alignment = 4;
        let final_size = (max_variant_size + union_alignment - 1) & !(union_alignment - 1);

        final_size
    }

    /// Returns which variant this tag union holds. Note that this never includes a payload!
    pub fn discriminant(&self) -> discriminant_InternalAttr {
        // SAFETY: this function makes assumptions on the memory layout which are not guaranteed
        // therefore may have undefined behaviour in some circumstances
        // The alignment of the captured type of the closure in unknown and so is the
        // alignment of this type. The alignment is  at least 8 and on wasm which is the
        // intended platform this is the largest reasonable alignment of the type therefore
        // it is a reasonable guess
        unsafe {
            let bytes = core::mem::transmute::<&Self, *const u8>(self);

            core::mem::transmute::<u8, discriminant_InternalAttr>(
                *bytes.add(Self::size() as usize - 8),
            )
        }
    }
}

impl core::fmt::Debug for InternalAttr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // use discriminant_InternalAttr::*;
        //
        //         unsafe {
        //             match self.discriminant {
        //                 Alt => {
        //                     let field: &roc_std::RocStr = &self.payload.Alt;
        //                     f.debug_tuple("InternalAttr::Alt").field(field).finish()
        //                 }
        //                 Attribute => {
        //                     let field: &InternalAttr_Attribute = &self.payload.Attribute;
        //                     f.debug_tuple("InternalAttr::Attribute")
        //                         .field(field)
        //                         .finish()
        //                 }
        //                 Autocomplete => {
        //                     let field: &roc_std::RocStr = &self.payload.Autocomplete;
        //                     f.debug_tuple("InternalAttr::Autocomplete")
        //                         .field(field)
        //                         .finish()
        //                 }
        //                 Checked => {
        //                     let field: &bool = &self.payload.Checked;
        //                     f.debug_tuple("InternalAttr::Checked").field(field).finish()
        //                 }
        //                 Class => {
        //                     let field: &roc_std::RocStr = &self.payload.Class;
        //                     f.debug_tuple("InternalAttr::Class").field(field).finish()
        //                 }
        //                 DataAttribute => {
        //                     let field: &InternalAttr_Attribute = &self.payload.DataAttribute;
        //                     f.debug_tuple("InternalAttr::DataAttribute")
        //                         .field(field)
        //                         .finish()
        //                 }
        //                 Disabled => {
        //                     let field: &bool = &self.payload.Disabled;
        //                     f.debug_tuple("InternalAttr::Disabled")
        //                         .field(field)
        //                         .finish()
        //                 }
        //                 Hidden => {
        //                     let field: &bool = &self.payload.Hidden;
        //                     f.debug_tuple("InternalAttr::Hidden").field(field).finish()
        //                 }
        //                 Href => {
        //                     let field: &roc_std::RocStr = &self.payload.Href;
        //                     f.debug_tuple("InternalAttr::Href").field(field).finish()
        //                 }
        //                 Id => {
        //                     let field: &roc_std::RocStr = &self.payload.Id;
        //                     f.debug_tuple("InternalAttr::Id").field(field).finish()
        //                 }
        //                 Multiple => {
        //                     let field: &bool = &self.payload.Multiple;
        //                     f.debug_tuple("InternalAttr::Multiple")
        //                         .field(field)
        //                         .finish()
        //                 }
        //                 Name => {
        //                     let field: &roc_std::RocStr = &self.payload.Name;
        //                     f.debug_tuple("InternalAttr::Name").field(field).finish()
        //                 }
        //                 OnEvent => {
        //                     let field: &InternalAttr_OnEvent = &self.payload.OnEvent;
        //                     f.debug_tuple("InternalAttr::OnEvent").field(field).finish()
        //                 }
        //                 Placeholder => {
        //                     let field: &roc_std::RocStr = &self.payload.Placeholder;
        //                     f.debug_tuple("InternalAttr::Placeholder")
        //                         .field(field)
        //                         .finish()
        //                 }
        //                 Readonly => {
        //                     let field: &bool = &self.payload.Readonly;
        //                     f.debug_tuple("InternalAttr::Readonly")
        //                         .field(field)
        //                         .finish()
        //                 }
        //                 Required => {
        //                     let field: &bool = &self.payload.Required;
        //                     f.debug_tuple("InternalAttr::Required")
        //                         .field(field)
        //                         .finish()
        //                 }
        //                 Selected => {
        //                     let field: &bool = &self.payload.Selected;
        //                     f.debug_tuple("InternalAttr::Selected")
        //                         .field(field)
        //                         .finish()
        //                 }
        //                 Src => {
        //                     let field: &roc_std::RocStr = &self.payload.Src;
        //                     f.debug_tuple("InternalAttr::Src").field(field).finish()
        //                 }
        //                 Style => {
        //                     let field: &roc_std::RocStr = &self.payload.Style;
        //                     f.debug_tuple("InternalAttr::Style").field(field).finish()
        //                 }
        //                 Tabindex => {
        //                     let field: &i32 = &self.payload.Tabindex;
        //                     f.debug_tuple("InternalAttr::Tabindex")
        //                         .field(field)
        //                         .finish()
        //                 }
        //                 Title => {
        //                     let field: &roc_std::RocStr = &self.payload.Title;
        //                     f.debug_tuple("InternalAttr::Title").field(field).finish()
        //                 }
        //                 Type => {
        //                     let field: &roc_std::RocStr = &self.payload.Type;
        //                     f.debug_tuple("InternalAttr::Type").field(field).finish()
        //                 }
        //                 Value => {
        //                     let field: &roc_std::RocStr = &self.payload.Value;
        //                     f.debug_tuple("InternalAttr::Value").field(field).finish()
        //                 }
        //             }
        //         }

        todo!()
    }
}

impl InternalAttr {
    // String variants
    pub fn borrow_Alt(&self) -> &roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Alt);
        unsafe { &(*(self as *const _ as *const union_InternalAttr)).Alt }
    }

    pub fn borrow_Autocomplete(&self) -> &roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Autocomplete);
        unsafe { &(*(self as *const _ as *const union_InternalAttr)).Autocomplete }
    }

    pub fn borrow_Class(&self) -> &roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Class);
        unsafe { &(*(self as *const _ as *const union_InternalAttr)).Class }
    }

    pub fn borrow_Href(&self) -> &roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Href);
        unsafe { &(*(self as *const _ as *const union_InternalAttr)).Href }
    }

    pub fn borrow_Id(&self) -> &roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Id);
        unsafe { &(*(self as *const _ as *const union_InternalAttr)).Id }
    }

    pub fn borrow_Name(&self) -> &roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Name);
        unsafe { &(*(self as *const _ as *const union_InternalAttr)).Name }
    }

    pub fn borrow_Placeholder(&self) -> &roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Placeholder);
        unsafe { &(*(self as *const _ as *const union_InternalAttr)).Placeholder }
    }

    pub fn borrow_Src(&self) -> &roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Src);
        unsafe { &(*(self as *const _ as *const union_InternalAttr)).Src }
    }

    pub fn borrow_Style(&self) -> &roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Style);
        unsafe { &(*(self as *const _ as *const union_InternalAttr)).Style }
    }

    pub fn borrow_Title(&self) -> &roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Title);
        unsafe { &(*(self as *const _ as *const union_InternalAttr)).Title }
    }

    pub fn borrow_Type(&self) -> &roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Type);
        unsafe { &(*(self as *const _ as *const union_InternalAttr)).Type }
    }

    pub fn borrow_Value(&self) -> &roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Value);
        unsafe { &(*(self as *const _ as *const union_InternalAttr)).Value }
    }

    // Boolean variants
    pub fn borrow_Checked(&self) -> bool {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Checked);
        unsafe { (*(self as *const _ as *const union_InternalAttr)).Checked }
    }

    pub fn borrow_Disabled(&self) -> bool {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Disabled);
        unsafe { (*(self as *const _ as *const union_InternalAttr)).Disabled }
    }

    pub fn borrow_Hidden(&self) -> bool {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Hidden);
        unsafe { (*(self as *const _ as *const union_InternalAttr)).Hidden }
    }

    pub fn borrow_Multiple(&self) -> bool {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Multiple);
        unsafe { (*(self as *const _ as *const union_InternalAttr)).Multiple }
    }

    pub fn borrow_Readonly(&self) -> bool {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Readonly);
        unsafe { (*(self as *const _ as *const union_InternalAttr)).Readonly }
    }

    pub fn borrow_Required(&self) -> bool {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Required);
        unsafe { (*(self as *const _ as *const union_InternalAttr)).Required }
    }

    pub fn borrow_Selected(&self) -> bool {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Selected);
        unsafe { (*(self as *const _ as *const union_InternalAttr)).Selected }
    }

    // Integer variant
    pub fn borrow_Tabindex(&self) -> i32 {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Tabindex);
        unsafe { (*(self as *const _ as *const union_InternalAttr)).Tabindex }
    }

    // Custom type variants
    pub fn borrow_Attribute(&self) -> &InternalAttr_Attribute {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Attribute);
        unsafe { &(*(self as *const _ as *const union_InternalAttr)).Attribute }
    }

    pub fn borrow_DataAttribute(&self) -> &InternalAttr_Attribute {
        debug_assert_eq!(
            self.discriminant(),
            discriminant_InternalAttr::DataAttribute
        );
        unsafe { &(*(self as *const _ as *const union_InternalAttr)).DataAttribute }
    }

    pub fn borrow_OnEvent(&self) -> &InternalAttr_OnEvent {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::OnEvent);
        unsafe { &(*(self as *const _ as *const union_InternalAttr)).OnEvent }
    }

    // Mutable variants - following the same pattern but returning mutable references

    // String variants (mutable)
    pub fn borrow_mut_Alt(&mut self) -> &mut roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Alt);
        unsafe { &mut (*(self as *mut _ as *mut union_InternalAttr)).Alt }
    }

    pub fn borrow_mut_Autocomplete(&mut self) -> &mut roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Autocomplete);
        unsafe { &mut (*(self as *mut _ as *mut union_InternalAttr)).Autocomplete }
    }

    pub fn borrow_mut_Class(&mut self) -> &mut roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Class);
        unsafe { &mut (*(self as *mut _ as *mut union_InternalAttr)).Class }
    }

    pub fn borrow_mut_Href(&mut self) -> &mut roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Href);
        unsafe { &mut (*(self as *mut _ as *mut union_InternalAttr)).Href }
    }

    pub fn borrow_mut_Id(&mut self) -> &mut roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Id);
        unsafe { &mut (*(self as *mut _ as *mut union_InternalAttr)).Id }
    }

    pub fn borrow_mut_Name(&mut self) -> &mut roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Name);
        unsafe { &mut (*(self as *mut _ as *mut union_InternalAttr)).Name }
    }

    pub fn borrow_mut_Placeholder(&mut self) -> &mut roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Placeholder);
        unsafe { &mut (*(self as *mut _ as *mut union_InternalAttr)).Placeholder }
    }

    pub fn borrow_mut_Src(&mut self) -> &mut roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Src);
        unsafe { &mut (*(self as *mut _ as *mut union_InternalAttr)).Src }
    }

    pub fn borrow_mut_Style(&mut self) -> &mut roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Style);
        unsafe { &mut (*(self as *mut _ as *mut union_InternalAttr)).Style }
    }

    pub fn borrow_mut_Title(&mut self) -> &mut roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Title);
        unsafe { &mut (*(self as *mut _ as *mut union_InternalAttr)).Title }
    }

    pub fn borrow_mut_Type(&mut self) -> &mut roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Type);
        unsafe { &mut (*(self as *mut _ as *mut union_InternalAttr)).Type }
    }

    pub fn borrow_mut_Value(&mut self) -> &mut roc_std::RocStr {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Value);
        unsafe { &mut (*(self as *mut _ as *mut union_InternalAttr)).Value }
    }

    // Boolean variants (mutable)
    pub fn borrow_mut_Checked(&mut self) -> &mut bool {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Checked);
        unsafe { &mut (*(self as *mut _ as *mut union_InternalAttr)).Checked }
    }

    pub fn borrow_mut_Disabled(&mut self) -> &mut bool {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Disabled);
        unsafe { &mut (*(self as *mut _ as *mut union_InternalAttr)).Disabled }
    }

    pub fn borrow_mut_Hidden(&mut self) -> &mut bool {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Hidden);
        unsafe { &mut (*(self as *mut _ as *mut union_InternalAttr)).Hidden }
    }

    pub fn borrow_mut_Multiple(&mut self) -> &mut bool {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Multiple);
        unsafe { &mut (*(self as *mut _ as *mut union_InternalAttr)).Multiple }
    }

    pub fn borrow_mut_Readonly(&mut self) -> &mut bool {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Readonly);
        unsafe { &mut (*(self as *mut _ as *mut union_InternalAttr)).Readonly }
    }

    pub fn borrow_mut_Required(&mut self) -> &mut bool {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Required);
        unsafe { &mut (*(self as *mut _ as *mut union_InternalAttr)).Required }
    }

    pub fn borrow_mut_Selected(&mut self) -> &mut bool {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Selected);
        unsafe { &mut (*(self as *mut _ as *mut union_InternalAttr)).Selected }
    }

    // Integer variant (mutable)
    pub fn borrow_mut_Tabindex(&mut self) -> &mut i32 {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Tabindex);
        unsafe { &mut (*(self as *mut _ as *mut union_InternalAttr)).Tabindex }
    }

    // Custom type variants (mutable)
    pub fn borrow_mut_Attribute(&mut self) -> &mut InternalAttr_Attribute {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::Attribute);
        unsafe { &mut (*(self as *mut _ as *mut union_InternalAttr)).Attribute }
    }

    pub fn borrow_mut_DataAttribute(&mut self) -> &mut InternalAttr_Attribute {
        debug_assert_eq!(
            self.discriminant(),
            discriminant_InternalAttr::DataAttribute
        );
        unsafe { &mut (*(self as *mut _ as *mut union_InternalAttr)).DataAttribute }
    }

    pub fn borrow_mut_OnEvent(&mut self) -> &mut InternalAttr_OnEvent {
        debug_assert_eq!(self.discriminant(), discriminant_InternalAttr::OnEvent);
        unsafe { &mut (*(self as *mut _ as *mut union_InternalAttr)).OnEvent }
    }
}

// impl Drop for InternalAttr {
//     fn drop(&mut self) {
//         // Drop the payloads
//         match self.discriminant() {
//             discriminant_InternalAttr::Alt => unsafe {
//                 core::mem::ManuallyDrop::drop(&mut self.payload.Alt)
//             },
//             discriminant_InternalAttr::Attribute => unsafe {
//                 core::mem::ManuallyDrop::drop(&mut self.payload.Attribute)
//             },
//             discriminant_InternalAttr::Autocomplete => unsafe {
//                 core::mem::ManuallyDrop::drop(&mut self.payload.Autocomplete)
//             },
//             discriminant_InternalAttr::Checked => {}
//             discriminant_InternalAttr::Class => unsafe {
//                 core::mem::ManuallyDrop::drop(&mut self.payload.Class)
//             },
//             discriminant_InternalAttr::DataAttribute => unsafe {
//                 core::mem::ManuallyDrop::drop(&mut self.payload.DataAttribute)
//             },
//             discriminant_InternalAttr::Disabled => {}
//             discriminant_InternalAttr::Hidden => {}
//             discriminant_InternalAttr::Href => unsafe {
//                 core::mem::ManuallyDrop::drop(&mut self.payload.Href)
//             },
//             discriminant_InternalAttr::Id => unsafe {
//                 core::mem::ManuallyDrop::drop(&mut self.payload.Id)
//             },
//             discriminant_InternalAttr::Multiple => {}
//             discriminant_InternalAttr::Name => unsafe {
//                 core::mem::ManuallyDrop::drop(&mut self.payload.Name)
//             },
//             discriminant_InternalAttr::OnEvent => unsafe {
//                 core::mem::ManuallyDrop::drop(&mut self.payload.OnEvent)
//             },
//             discriminant_InternalAttr::Placeholder => unsafe {
//                 core::mem::ManuallyDrop::drop(&mut self.payload.Placeholder)
//             },
//             discriminant_InternalAttr::Readonly => {}
//             discriminant_InternalAttr::Required => {}
//             discriminant_InternalAttr::Selected => {}
//             discriminant_InternalAttr::Src => unsafe {
//                 core::mem::ManuallyDrop::drop(&mut self.payload.Src)
//             },
//             discriminant_InternalAttr::Style => unsafe {
//                 core::mem::ManuallyDrop::drop(&mut self.payload.Style)
//             },
//             discriminant_InternalAttr::Tabindex => {}
//             discriminant_InternalAttr::Title => unsafe {
//                 core::mem::ManuallyDrop::drop(&mut self.payload.Title)
//             },
//             discriminant_InternalAttr::Type => unsafe {
//                 core::mem::ManuallyDrop::drop(&mut self.payload.Type)
//             },
//             discriminant_InternalAttr::Value => unsafe {
//                 core::mem::ManuallyDrop::drop(&mut self.payload.Value)
//             },
//         }
//     }
// }

impl roc_std::RocRefcounted for InternalAttr {
    fn inc(&mut self) {
        unimplemented!();
    }
    fn dec(&mut self) {
        unimplemented!();
    }
    fn is_refcounted() -> bool {
        true
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct InternalHtmlElementFields {
    pub attrs: roc_std::RocList<InternalAttr>,
    pub children: roc_std::RocList<InternalHtml>,
    pub tag: roc_std::RocStr,
}

impl roc_std::RocRefcounted for InternalHtmlElementFields {
    fn inc(&mut self) {
        self.attrs.inc();
        self.children.inc();
        self.tag.inc();
    }
    fn dec(&mut self) {
        self.attrs.dec();
        self.children.dec();
        self.tag.dec();
    }
    fn is_refcounted() -> bool {
        true
    }
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct InternalHtml_Element {
    pub fields: InternalHtmlElementFields,
}

impl roc_std::RocRefcounted for InternalHtml_Element {
    fn inc(&mut self) {
        self.fields.inc();
    }
    fn dec(&mut self) {
        self.fields.dec();
    }
    fn is_refcounted() -> bool {
        true
    }
}

#[derive(Clone, Default, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[repr(transparent)]
pub struct InternalHtml_Text {
    pub f0: roc_std::RocStr,
}

impl roc_std::RocRefcounted for InternalHtml_Text {
    fn inc(&mut self) {
        self.f0.inc();
    }
    fn dec(&mut self) {
        self.f0.dec();
    }
    fn is_refcounted() -> bool {
        true
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[repr(u8)]
pub enum discriminant_InternalHtml {
    Element = 0,
    Text = 1,
}

impl core::fmt::Debug for discriminant_InternalHtml {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Element => f.write_str("discriminant_InternalHtml::Element"),
            Self::Text => f.write_str("discriminant_InternalHtml::Text"),
        }
    }
}

roc_refcounted_noop_impl!(discriminant_InternalHtml);

#[repr(transparent)]
pub struct InternalHtml(*mut union_InternalHtml);

impl InternalHtml {
    pub fn discriminant(&self) -> discriminant_InternalHtml {
        let discriminants = {
            use discriminant_InternalHtml::*;

            [Element, Text]
        };

        if self.0.is_null() {
            unreachable!("this pointer cannot be NULL")
        } else {
            match std::mem::size_of::<usize>() {
                4 => discriminants[self.0 as usize & 0b011],
                8 => discriminants[self.0 as usize & 0b111],
                _ => unreachable!(),
            }
        }
    }

    fn unmasked_pointer(&self) -> *mut union_InternalHtml {
        debug_assert!(!self.0.is_null());

        let mask = match std::mem::size_of::<usize>() {
            4 => !0b011usize,
            8 => !0b111usize,
            _ => unreachable!(),
        };

        ((self.0 as usize) & mask) as *mut union_InternalHtml
    }

    unsafe fn ptr_read_union(&self) -> core::mem::ManuallyDrop<union_InternalHtml> {
        let ptr = self.unmasked_pointer();

        core::mem::ManuallyDrop::new(unsafe { std::ptr::read(ptr) })
    }

    pub fn is_Element(&self) -> bool {
        matches!(self.discriminant(), discriminant_InternalHtml::Element)
    }

    pub fn Element(f0: InternalHtmlElementFields) -> Self {
        let tag_id = discriminant_InternalHtml::Element;

        let payload = InternalHtml_Element { fields: f0 };

        let union_payload = union_InternalHtml {
            Element: core::mem::ManuallyDrop::new(payload),
        };

        let ptr = unsafe { roc_std::RocBox::leak(roc_std::RocBox::new(union_payload)) };

        Self((ptr as usize | tag_id as usize) as *mut _)
    }

    pub fn get_Element_fields(&self) -> &InternalHtmlElementFields {
        debug_assert!(self.is_Element());
        unsafe { &*self.unmasked_pointer().cast() }
    }

    pub fn get_Element_fields_mut(&mut self) -> &mut InternalHtmlElementFields {
        debug_assert!(self.is_Element());
        unsafe { &mut *self.unmasked_pointer().cast() }
    }

    pub fn get_Element(mut self) -> InternalHtml_Element {
        debug_assert!(self.is_Element());

        unsafe { core::mem::ManuallyDrop::take(&mut self.ptr_read_union().Element) }
    }

    pub fn is_Text(&self) -> bool {
        matches!(self.discriminant(), discriminant_InternalHtml::Text)
    }

    pub fn Text(f0: roc_std::RocStr) -> Self {
        let tag_id = discriminant_InternalHtml::Text;

        let payload = InternalHtml_Text { f0 };

        let union_payload = union_InternalHtml {
            Text: core::mem::ManuallyDrop::new(payload),
        };

        let ptr = unsafe { roc_std::RocBox::leak(roc_std::RocBox::new(union_payload)) };

        Self((ptr as usize | tag_id as usize) as *mut _)
    }

    pub fn get_Text_f0(&self) -> &roc_std::RocStr {
        debug_assert!(self.is_Text());

        // extern "C" {
        //     fn foobar(tag_id: u16, field_index: usize) -> usize;
        // }

        // let offset = unsafe { foobar(0) };
        let offset = 0;
        unsafe { &*self.unmasked_pointer().add(offset).cast() }
    }

    pub fn get_Text(mut self) -> InternalHtml_Text {
        debug_assert!(self.is_Text());

        unsafe { core::mem::ManuallyDrop::take(&mut self.ptr_read_union().Text) }
    }
}

impl Clone for InternalHtml {
    fn clone(&self) -> Self {
        use discriminant_InternalHtml::*;

        let discriminant = self.discriminant();

        match discriminant {
            Element => {
                let tag_id = discriminant_InternalHtml::Element;

                let payload_union = unsafe { self.ptr_read_union() };
                let payload = union_InternalHtml {
                    Element: unsafe { payload_union.Element.clone() },
                };

                let ptr = unsafe { roc_std::RocBox::leak(roc_std::RocBox::new(payload)) };

                Self((ptr as usize | tag_id as usize) as *mut _)
            }
            Text => {
                let tag_id = discriminant_InternalHtml::Text;

                let payload_union = unsafe { self.ptr_read_union() };
                let payload = union_InternalHtml {
                    Text: unsafe { payload_union.Text.clone() },
                };

                let ptr = unsafe { roc_std::RocBox::leak(roc_std::RocBox::new(payload)) };

                Self((ptr as usize | tag_id as usize) as *mut _)
            }
        }
    }
}

impl core::fmt::Debug for InternalHtml {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use discriminant_InternalHtml::*;

        match self.discriminant() {
            Element => {
                let payload_union = unsafe { self.ptr_read_union() };

                unsafe {
                    f.debug_tuple("InternalHtml::Element")
                        .field(&payload_union.Element.fields)
                        .finish()
                }
            }
            Text => {
                let payload_union = unsafe { self.ptr_read_union() };

                unsafe {
                    f.debug_tuple("InternalHtml::Text")
                        .field(&payload_union.Text.f0)
                        .finish()
                }
            }
        }
    }
}

#[repr(C)]
union union_InternalHtml {
    Element: core::mem::ManuallyDrop<InternalHtml_Element>,
    Text: core::mem::ManuallyDrop<InternalHtml_Text>,
}

impl roc_std::RocRefcounted for InternalHtml {
    fn inc(&mut self) {
        unimplemented!();
    }
    fn dec(&mut self) {
        unimplemented!();
    }
    fn is_refcounted() -> bool {
        true
    }
}

impl roc_std::RocRefcounted for union_InternalHtml {
    fn inc(&mut self) {
        unimplemented!();
    }
    fn dec(&mut self) {
        unimplemented!();
    }
    fn is_refcounted() -> bool {
        true
    }
}

pub fn frontend_init_for_host(arg0: u32) -> RocBox<()> {
    extern "C" {
        fn roc__frontend_init_for_host_1_exposed_generic(_: *mut RocBox<()>, _: u32);
    }

    let mut ret = core::mem::MaybeUninit::uninit();

    unsafe {
        roc__frontend_init_for_host_1_exposed_generic(ret.as_mut_ptr(), arg0);

        ret.assume_init()
    }
}

pub fn frontend_view_for_host(model: RocBox<()>) -> InternalHtml {
    extern "C" {
        fn roc__frontend_view_for_host_1_exposed_generic(_: *mut InternalHtml, _: RocBox<()>);
    }

    let mut ret = core::mem::MaybeUninit::uninit();

    unsafe {
        roc__frontend_view_for_host_1_exposed_generic(ret.as_mut_ptr(), model);

        ret.assume_init()
    }
}

pub fn frontend_update_for_host(model: RocBox<()>, boxed_msg: RocBox<()>) -> R1 {
    extern "C" {
        fn roc__frontend_update_for_host_1_exposed_generic(
            ret: *mut R1,
            model: RocBox<()>,
            msg: RocBox<()>,
        );
    }

    let mut ret = core::mem::MaybeUninit::uninit();

    unsafe {
        roc__frontend_update_for_host_1_exposed_generic(ret.as_mut_ptr(), model, boxed_msg);

        ret.assume_init()
    }
}
