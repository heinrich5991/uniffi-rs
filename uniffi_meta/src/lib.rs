/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{collections::BTreeMap, hash::Hasher};
pub use uniffi_checksum_derive::Checksum;

use serde::Serialize;

mod ffi_names;
pub use ffi_names::*;

mod group;
pub use group::{group_metadata, MetadataGroup};

mod reader;
pub use reader::{read_metadata, read_metadata_type};

mod types;
pub use types::{AsType, ExternalKind, ObjectImpl, Type, TypeIterator};

// This needs to match the minor version of the `uniffi` crate.  See
// `docs/uniffi-versioning.md` for details.
//
// Once we get to 1.0, then we'll need to update the scheme to something like 100 + major_version
pub const UNIFFI_CONTRACT_VERSION: u32 = 22;

/// Similar to std::hash::Hash.
///
/// Implementations of this trait are expected to update the hasher state in
/// the same way across platforms. #[derive(Checksum)] will do the right thing.
pub trait Checksum {
    fn checksum<H: Hasher>(&self, state: &mut H);
}

impl Checksum for bool {
    fn checksum<H: Hasher>(&self, state: &mut H) {
        state.write_u8(*self as u8);
    }
}

impl Checksum for u64 {
    fn checksum<H: Hasher>(&self, state: &mut H) {
        state.write(&self.to_le_bytes());
    }
}

impl Checksum for i64 {
    fn checksum<H: Hasher>(&self, state: &mut H) {
        state.write(&self.to_le_bytes());
    }
}

impl<T: Checksum> Checksum for Box<T> {
    fn checksum<H: Hasher>(&self, state: &mut H) {
        (**self).checksum(state)
    }
}

impl<T: Checksum> Checksum for [T] {
    fn checksum<H: Hasher>(&self, state: &mut H) {
        state.write(&(self.len() as u64).to_le_bytes());
        for item in self {
            Checksum::checksum(item, state);
        }
    }
}

impl<T: Checksum> Checksum for Vec<T> {
    fn checksum<H: Hasher>(&self, state: &mut H) {
        Checksum::checksum(&**self, state);
    }
}

impl<K: Checksum, V: Checksum> Checksum for BTreeMap<K, V> {
    fn checksum<H: Hasher>(&self, state: &mut H) {
        state.write(&(self.len() as u64).to_le_bytes());
        for (key, value) in self {
            Checksum::checksum(key, state);
            Checksum::checksum(value, state);
        }
    }
}

impl<T: Checksum> Checksum for Option<T> {
    fn checksum<H: Hasher>(&self, state: &mut H) {
        match self {
            None => state.write(&0u64.to_le_bytes()),
            Some(value) => {
                state.write(&1u64.to_le_bytes());
                Checksum::checksum(value, state)
            }
        }
    }
}

impl Checksum for str {
    fn checksum<H: Hasher>(&self, state: &mut H) {
        state.write(self.as_bytes());
        state.write_u8(0xff);
    }
}

impl Checksum for String {
    fn checksum<H: Hasher>(&self, state: &mut H) {
        (**self).checksum(state)
    }
}

impl Checksum for &str {
    fn checksum<H: Hasher>(&self, state: &mut H) {
        (**self).checksum(state)
    }
}

// The namespace of a Component interface.
//
// This is used to match up the macro metadata with the UDL items.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct NamespaceMetadata {
    pub crate_name: String,
    pub name: String,
}

// UDL file included with `include_scaffolding!()`
//
// This is to find the UDL files in library mode generation
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct UdlFile {
    pub module_path: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct FnMetadata {
    pub module_path: String,
    pub name: String,
    pub is_async: bool,
    pub inputs: Vec<FnParamMetadata>,
    pub return_type: Option<Type>,
    pub throws: Option<Type>,
    pub checksum: Option<u16>,
}

impl FnMetadata {
    pub fn ffi_symbol_name(&self) -> String {
        fn_symbol_name(&self.module_path, &self.name)
    }

    pub fn checksum_symbol_name(&self) -> String {
        fn_checksum_symbol_name(&self.module_path, &self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ConstructorMetadata {
    pub module_path: String,
    pub self_name: String,
    pub name: String,
    pub inputs: Vec<FnParamMetadata>,
    pub throws: Option<Type>,
    pub checksum: Option<u16>,
}

impl ConstructorMetadata {
    pub fn ffi_symbol_name(&self) -> String {
        constructor_symbol_name(&self.module_path, &self.self_name, &self.name)
    }

    pub fn checksum_symbol_name(&self) -> String {
        constructor_checksum_symbol_name(&self.module_path, &self.self_name, &self.name)
    }

    pub fn is_primary(&self) -> bool {
        self.name == "new"
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct MethodMetadata {
    pub module_path: String,
    pub self_name: String,
    pub name: String,
    pub is_async: bool,
    pub inputs: Vec<FnParamMetadata>,
    pub return_type: Option<Type>,
    pub throws: Option<Type>,
    pub takes_self_by_arc: bool, // unused except by rust udl bindgen.
    pub checksum: Option<u16>,
}

impl MethodMetadata {
    pub fn ffi_symbol_name(&self) -> String {
        method_symbol_name(&self.module_path, &self.self_name, &self.name)
    }

    pub fn checksum_symbol_name(&self) -> String {
        method_checksum_symbol_name(&self.module_path, &self.self_name, &self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct TraitMethodMetadata {
    pub module_path: String,
    pub trait_name: String,
    // Note: the position of `index` is important since it causes callback interface methods to be
    // ordered correctly in MetadataGroup.items
    pub index: u32,
    pub name: String,
    pub is_async: bool,
    pub inputs: Vec<FnParamMetadata>,
    pub return_type: Option<Type>,
    pub throws: Option<Type>,
    pub takes_self_by_arc: bool, // unused except by rust udl bindgen.
    pub checksum: Option<u16>,
}

impl TraitMethodMetadata {
    pub fn ffi_symbol_name(&self) -> String {
        method_symbol_name(&self.module_path, &self.trait_name, &self.name)
    }

    pub fn checksum_symbol_name(&self) -> String {
        method_checksum_symbol_name(&self.module_path, &self.trait_name, &self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct FnParamMetadata {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: Type,
    pub by_ref: bool,
    pub optional: bool,
    pub default: Option<LiteralMetadata>,
}

impl FnParamMetadata {
    pub fn simple(name: &str, ty: Type) -> Self {
        Self {
            name: name.to_string(),
            ty,
            by_ref: false,
            optional: false,
            default: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Checksum)]
pub enum LiteralMetadata {
    Boolean(bool),
    String(String),
    // Integers are represented as the widest representation we can.
    // Number formatting vary with language and radix, so we avoid a lot of parsing and
    // formatting duplication by using only signed and unsigned variants.
    UInt(u64, Radix, Type),
    Int(i64, Radix, Type),
    // Pass the string representation through as typed in the UDL.
    // This avoids a lot of uncertainty around precision and accuracy,
    // though bindings for languages less sophisticated number parsing than WebIDL
    // will have to do extra work.
    Float(String, Type),
    Enum(String, Type),
    EmptySequence,
    EmptyMap,
    Null,
}

// Represent the radix of integer literal values.
// We preserve the radix into the generated bindings for readability reasons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Checksum)]
pub enum Radix {
    Decimal = 10,
    Octal = 8,
    Hexadecimal = 16,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct RecordMetadata {
    pub module_path: String,
    pub name: String,
    pub fields: Vec<FieldMetadata>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct FieldMetadata {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: Type,
    pub default: Option<LiteralMetadata>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct EnumMetadata {
    pub module_path: String,
    pub name: String,
    pub variants: Vec<VariantMetadata>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct VariantMetadata {
    pub name: String,
    pub fields: Vec<FieldMetadata>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ObjectMetadata {
    pub module_path: String,
    pub name: String,
    pub imp: types::ObjectImpl,
    pub uniffi_traits: Vec<UniffiTraitMetadata>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct CallbackInterfaceMetadata {
    pub module_path: String,
    pub name: String,
}

impl ObjectMetadata {
    /// FFI symbol name for the `free` function for this object.
    ///
    /// This function is used to free the memory used by this object.
    pub fn free_ffi_symbol_name(&self) -> String {
        free_fn_symbol_name(&self.module_path, &self.name)
    }
}

/// The list of traits we support generating helper methods for.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum UniffiTraitMetadata {
    Debug {
        fmt: MethodMetadata,
    },
    Display {
        fmt: MethodMetadata,
    },
    Eq {
        eq: MethodMetadata,
        ne: MethodMetadata,
    },
    Hash {
        hash: MethodMetadata,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum ErrorMetadata {
    Enum { enum_: EnumMetadata, is_flat: bool },
}

impl ErrorMetadata {
    pub fn name(&self) -> &String {
        match self {
            Self::Enum { enum_, .. } => &enum_.name,
        }
    }

    pub fn module_path(&self) -> &String {
        match self {
            Self::Enum { enum_, .. } => &enum_.module_path,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct CustomTypeMetadata {
    pub module_path: String,
    pub name: String,
    pub builtin: Type,
}

/// Returns the last 16 bits of the value's hash as computed with [`SipHasher13`].
///
/// This is used as a safeguard against different UniFFI versions being used for scaffolding and
/// bindings generation.
pub fn checksum<T: Checksum>(val: &T) -> u16 {
    let mut hasher = siphasher::sip::SipHasher13::new();
    val.checksum(&mut hasher);
    (hasher.finish() & 0x000000000000FFFF) as u16
}

/// Enum covering all the possible metadata types
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Metadata {
    Namespace(NamespaceMetadata),
    UdlFile(UdlFile),
    Func(FnMetadata),
    Object(ObjectMetadata),
    CallbackInterface(CallbackInterfaceMetadata),
    Record(RecordMetadata),
    Enum(EnumMetadata),
    Error(ErrorMetadata),
    Constructor(ConstructorMetadata),
    Method(MethodMetadata),
    TraitMethod(TraitMethodMetadata),
    CustomType(CustomTypeMetadata),
}

impl Metadata {
    pub fn read(data: &[u8]) -> anyhow::Result<Self> {
        read_metadata(data)
    }
}

impl From<NamespaceMetadata> for Metadata {
    fn from(value: NamespaceMetadata) -> Metadata {
        Self::Namespace(value)
    }
}

impl From<UdlFile> for Metadata {
    fn from(value: UdlFile) -> Metadata {
        Self::UdlFile(value)
    }
}

impl From<FnMetadata> for Metadata {
    fn from(value: FnMetadata) -> Metadata {
        Self::Func(value)
    }
}

impl From<ConstructorMetadata> for Metadata {
    fn from(c: ConstructorMetadata) -> Self {
        Self::Constructor(c)
    }
}

impl From<MethodMetadata> for Metadata {
    fn from(m: MethodMetadata) -> Self {
        Self::Method(m)
    }
}

impl From<RecordMetadata> for Metadata {
    fn from(r: RecordMetadata) -> Self {
        Self::Record(r)
    }
}

impl From<EnumMetadata> for Metadata {
    fn from(e: EnumMetadata) -> Self {
        Self::Enum(e)
    }
}

impl From<ErrorMetadata> for Metadata {
    fn from(e: ErrorMetadata) -> Self {
        Self::Error(e)
    }
}

impl From<ObjectMetadata> for Metadata {
    fn from(v: ObjectMetadata) -> Self {
        Self::Object(v)
    }
}

impl From<CallbackInterfaceMetadata> for Metadata {
    fn from(v: CallbackInterfaceMetadata) -> Self {
        Self::CallbackInterface(v)
    }
}

impl From<TraitMethodMetadata> for Metadata {
    fn from(v: TraitMethodMetadata) -> Self {
        Self::TraitMethod(v)
    }
}

impl From<CustomTypeMetadata> for Metadata {
    fn from(v: CustomTypeMetadata) -> Self {
        Self::CustomType(v)
    }
}
