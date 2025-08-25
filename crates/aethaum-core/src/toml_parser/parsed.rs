use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;
use crate::toml_parser::raw::{RawComponent, RawComponentField, RawComponentFile, RawEntityProto, RawEntityProtoFile, RawEvent, RawEventField, RawEventFile, RawSystem, RawSystemEventHandler, RawSystemFile, RawSystemNormal, RawSystemQuery, RawSystemUpdate, RawTomlCodeFile, RawWorld, RawWorldBuild, RawWorldCargo, RawWorldModules, RawWorldNormal};
use smart_string::SmartString;
use std::time::Duration;
use anyhow::Error;
use itertools::Itertools;
use one_or_many::OneOrMany;
use proc_macro2::Span;
use syn::Ident;

#[derive(Debug,Clone,PartialEq)]
pub enum LuaScript {
    Embed(SmartString),
    File(PathBuf),
}
impl LuaScript {
    pub fn is_embed(&self) -> bool {
        matches!(self, LuaScript::Embed(_))
    }
    pub fn is_file(&self) -> bool {
        matches!(self, LuaScript::File(_))
    }
    pub fn from_embed_or_file(embed: Option<SmartString>, file: Option<PathBuf>) -> Result<Option<Self>, anyhow::Error> {
        match (embed, file) {
            (Some(embed), None) => Ok(Some(LuaScript::Embed(embed))),
            (None, Some(file)) => Ok(Some(LuaScript::File(file))),
            (Some(_), Some(_)) => anyhow::bail!("Embed and File cannot be specified at the same time"),
            (None, None) => Ok(None),
        }
     }
}
pub trait TomlCode: Sized { //标记Trait, 用于约束Parser泛型
    type RawFile: RawTomlCodeFile + for<'de> serde::Deserialize<'de>;
    fn from_raw_file(raw: <Self::RawFile as RawTomlCodeFile>::RawPieces) -> Result<OneOrMany<Self>, anyhow::Error>;

}
pub trait Field {
    fn name_as_rust_ident(&self) -> Ident;
    fn type_as_rust_ident(&self) -> Ident;
}
pub trait Describable {
    fn description(&self) -> Option<&str> {
        None
    }
    fn field_description(&self) -> Option<impl Iterator<Item = (&str, &str)>>; //(字段名，字段描述)
}

pub trait AethaumRef {
    fn to_global_ref(self, module_name: SmartString) -> Self;
    fn to_local_ref(self) -> Self;
}

//Type Definition
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum PrimitiveType {
    Float,
    Int,
    Bool,
    Str,
    //TODO: 添加更多类型
}
impl std::fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveType::Float => write!(f, "float"),
            PrimitiveType::Int => write!(f, "int"),
            PrimitiveType::Bool => write!(f, "bool"),
            PrimitiveType::Str => write!(f, "str"),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum AethaumType {
    Primitive(PrimitiveType),
    Custom(SmartString)
}
impl std::fmt::Display for AethaumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AethaumType::Primitive(primitive) => write!(f, "{}", primitive),
            AethaumType::Custom(custom) => write!(f, "{}", custom),
        }
    }
}

impl AethaumType {
    pub fn is_primitive(&self) -> bool {
        matches!(self, AethaumType::Primitive(_))
    }
    pub fn is_custom(&self) -> bool {
        matches!(self, AethaumType::Custom(_))
    }
    pub fn from_type_str(type_str: &str) -> AethaumType {
        match type_str {
            "float" => AethaumType::Primitive(PrimitiveType::Float),
            "int" => AethaumType::Primitive(PrimitiveType::Int),
            "bool" => AethaumType::Primitive(PrimitiveType::Bool),
            "str" => AethaumType::Primitive(PrimitiveType::Str),

            _ => AethaumType::Custom(type_str.into()),
        }
    }
    pub fn to_rust_type(&self) -> Ident {
        match self {
            AethaumType::Primitive(primitive) => match primitive {
                PrimitiveType::Float => Ident::new("f64", Span::call_site()),
                PrimitiveType::Int => Ident::new("i64", Span::call_site()),
                PrimitiveType::Bool => Ident::new("bool", Span::call_site()),
                PrimitiveType::Str => Ident::new("String", Span::call_site()),
            },
            AethaumType::Custom(custom) => Ident::new(custom, Span::call_site()),
        }
    }
}

//Component
#[derive(Debug,PartialEq, Clone)]
pub struct ComponentField {
    pub name: SmartString,
    pub type_spec : AethaumType,
    pub default_value: Option<toml::Value>,
    pub description: Option<SmartString>
}

#[derive(Debug,PartialEq, Clone)]
pub struct Component {
    pub name: SmartString,
    pub description: Option<SmartString>,
    pub fields: Option<Vec<ComponentField>>
}
#[derive(Debug,PartialEq,Clone,Eq,Hash)]
pub struct ComponentRef {
    pub name: SmartString,
    pub module_name: Option<SmartString>,
}
impl ComponentRef {
    pub fn new(module_name: Option<impl Into<SmartString>>, name: impl Into<SmartString>) -> Self {
        Self { name: name.into() , module_name: module_name.map(|s| s.into())}
    }
    pub fn as_path_str(&self) -> String {
        match &self.module_name {
            None => self.name.to_string(),
            Some(module_name) => format!("{}::{}", module_name, self.name)
        }
    }
}
impl From<(&str, &str)> for ComponentRef {
    fn from((module_name,name): (&str,&str)) -> Self {
        Self::new(Some(module_name), name)
    }
}
impl From<(SmartString, SmartString)> for ComponentRef {
    fn from((module_name, name): (SmartString,SmartString)) -> Self {
        Self::new(Some(module_name), name)
    }
}
impl TryFrom<SmartString> for ComponentRef {
    type Error = anyhow::Error;
    fn try_from(s: SmartString) -> Result<Self, Self::Error> {
        let res : Vec<&str> = s.split("::").collect();
        match res.len() {
            0 => anyhow::bail!("Invalid component ref: {}, component name is required", s),
            1 => Ok(Self::new(None::<SmartString>, s)),
            2 => Ok(Self::new(Some(res[0]), res[1])),
            _ => anyhow::bail!("Invalid component ref: {}, nested module is currently not allowed", s),
        }
    }
}
impl std::fmt::Display for ComponentRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_path_str())
    }
}
#[derive(Debug, PartialEq, Clone)] //TODO: check if there is any conflict when construct
pub struct ComponentConstraint {
    include: Option<Vec<ComponentRef>>, //必须包含的组件
    exclude: Option<Vec<ComponentRef>>, //必须不包含的组件
}
impl ComponentConstraint {
    pub fn new_empty() -> Self {
        Self {
            include: Some(vec![]),
            exclude: Some(vec![]),
        }
    }
    pub fn with_include(mut self, include: Vec<ComponentRef>) -> Self {
        self.include = Some(include);
        self
    }
    pub fn with_exclude(mut self, exclude: Vec<ComponentRef>) -> Self {
        self.exclude = Some(exclude);
        self
    }
    pub fn chained_iter(&self) -> impl Iterator<Item = &ComponentRef> { //TODO: test it
        self.include.iter().flatten().chain(self.exclude.iter().flatten())
    }
    pub fn get_include(&self) -> Option<&Vec<ComponentRef>> {
        self.include.as_ref()
    }
    pub fn get_exclude(&self) -> Option<&Vec<ComponentRef>> {
        self.exclude.as_ref()
    }
}
impl TryFrom<(Option<Vec<SmartString>>, Option<Vec<SmartString>>)> for ComponentConstraint {
    type Error = anyhow::Error;
    fn try_from(
        (include, exclude): (Option<Vec<SmartString>>, Option<Vec<SmartString>>),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            include: include.map(|v| v.into_iter().map(|s| s.try_into()).try_collect()).transpose()?,
            exclude: exclude.map(|v| v.into_iter().map(|s| s.try_into()).try_collect()).transpose()?,
        })
    }
}

//Event
#[derive(Debug,PartialEq,Clone)]
pub struct EventField {
    pub name: SmartString,
    pub type_spec : AethaumType,
    pub description: Option<SmartString>
}

#[derive(Debug,PartialEq,Clone)]
pub struct Event {
    pub name: SmartString,
    pub description: Option<SmartString>,
    pub fields: Option<Vec<EventField>>
}
#[derive(Debug,PartialEq,Clone, Eq, Hash)]
pub struct EventRef {
    pub name: SmartString,
    pub module_name: Option<SmartString>,
}
impl EventRef {
    pub fn new(module_name: Option<impl Into<SmartString>>,name: impl Into<SmartString>) -> Self {
        Self { name: name.into() , module_name: module_name.map(|s| s.into())}
    }
    pub fn as_path_str(&self) -> String {
        match &self.module_name {
            None => self.name.to_string(),
            Some(module_name) => format!("{}::{}", module_name, self.name)
        }
    }
}
impl From<(&str,&str)> for EventRef {
    fn from((module_name,name): (&str, &str)) -> Self {
        Self::new(Some(module_name), name)
    }
}
impl From<(SmartString, SmartString)> for EventRef {
    fn from((module_name, name): (SmartString, SmartString)) -> Self {
        Self::new(Some(module_name), name)
    }
}
impl TryFrom<SmartString> for EventRef {
    type Error = anyhow::Error;
    fn try_from(s: SmartString) -> Result<Self, Self::Error> {
        let res : Vec<&str> = s.split("::").collect();
        match res.len() {
            0 => anyhow::bail!("Invalid event ref: {}, event name is required", s),
            1 => Ok(Self::new(None::<SmartString>, s)),
            2 => Ok(Self::new(Some(res[0]), res[1])),
            _ => anyhow::bail!("Invalid event ref: {}, nested module is currently not allowed", s),
        }
    }
}
impl std::fmt::Display for EventRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_path_str())
    }
}
//Entity Protos
#[derive(Debug,PartialEq,Clone)]
pub struct EntityProto {
    pub name: SmartString,
    pub description: Option<SmartString>,
    pub components: Vec<ComponentRef>
}
#[derive(Debug,PartialEq,Clone, Eq, Hash)]
pub struct EntityProtoRef {
    pub name: SmartString,
    pub module_name: Option<SmartString>,
}
impl EntityProtoRef {
    pub fn new(module_name: Option<impl Into<SmartString>>,name: impl Into<SmartString>) -> Self {
        Self { name: name.into(), module_name: module_name.map(|s| s.into()) }
    }
    pub fn as_path_str(&self) -> String {
        match &self.module_name {
            None => self.name.to_string(),
            Some(module_name) => format!("{}::{}", module_name, self.name)
        }
    }
}
impl From<(&str,&str)> for EntityProtoRef {
    fn from((module_name, name): (&str, &str)) -> Self {
        Self::new(Some(module_name), name)
    }
}
impl From<(SmartString, SmartString)> for EntityProtoRef {
    fn from((module_name, name): (SmartString, SmartString)) -> Self {
        Self::new(Some(module_name), name)
    }
}
impl TryFrom<SmartString> for EntityProtoRef {
    type Error = anyhow::Error;
    fn try_from(s: SmartString) -> Result<Self, Self::Error> {
        let res : Vec<&str> = s.split("::").collect();
        match res.len() {
            0 => anyhow::bail!("Invalid entity proto ref: {}, entity proto name is required", s),
            1 => Ok(Self::new(None::<SmartString>, s)),
            2 => Ok(Self::new(Some(res[0]), res[1])),
            _ => anyhow::bail!("Invalid entity proto ref: {}, nested module is currently not allowed", s)
        }
    }
}
impl std::fmt::Display for EntityProtoRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_path_str())
    }
}
//System
pub type SystemNormal = RawSystemNormal;
#[derive(Debug,PartialEq,Clone)]
pub struct SystemQuery {
    pub name: SmartString,
    pub description: Option<SmartString>,
    pub component_constraint: ComponentConstraint
}
#[derive(Debug,PartialEq,Clone)]
pub struct SystemEventHandler {
    pub watch_for: EventRef,
    pub priority: u32,
    pub logic: Option<LuaScript>
}
#[derive(Debug,PartialEq,Clone)]
pub struct SystemUpdate {
    pub interval: Duration,
    pub condition: Option<LuaScript>,
    pub logic: Option<LuaScript>
}
#[derive(Debug,PartialEq,Clone)]
pub struct System {
    pub normal: SystemNormal,
    pub queries: Vec<SystemQuery>,
    pub update: Option<SystemUpdate>,
    pub event_handlers: Vec<SystemEventHandler>
}
#[derive(Debug,PartialEq,Clone, Eq, Hash)]
pub struct SystemRef {
    pub name: SmartString,
    pub module_name: Option<SmartString>,
}
impl SystemRef {
    pub fn new(module_name: Option<impl Into<SmartString>>,name: impl Into<SmartString>) -> Self {
        Self { name: name.into(), module_name: module_name.map(|s| s.into()) }
    }
    pub fn as_path_str(&self) -> String {
        match &self.module_name {
            None => self.name.to_string(),
            Some(module_name) => format!("{}::{}", module_name, self.name)
        }
    }
}
impl From<(&str,&str)> for SystemRef {
    fn from((module_name, name): (&str, &str)) -> Self {
        Self::new(Some(module_name), name)
    }
}
impl From<(SmartString, SmartString)> for SystemRef {
    fn from((module_name, name): (SmartString, SmartString)) -> Self {
        Self::new(Some(module_name), name)
    }
}
impl TryFrom<SmartString> for SystemRef {
    type Error = anyhow::Error;
    fn try_from(s: SmartString) -> Result<Self, Self::Error> {
        let res : Vec<&str> = s.split("::").collect();
        match res.len() {
            0 => anyhow::bail!("Invalid system ref: {}, system name is required", s),
            1 => Ok(Self::new(None::<SmartString>, s)),
            2 => Ok(Self::new(Some(res[0]), res[1])),
            _ => anyhow::bail!("Invalid system ref: {}, nested module is currently not allowed", s)
        }
    }
}
impl std::fmt::Display for SystemRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
//TomlCode Mark
impl TomlCode for Component {
    type RawFile = RawComponentFile;
    fn from_raw_file(raw: <Self::RawFile as RawTomlCodeFile>::RawPieces) -> Result<OneOrMany<Self>, Error> {
        Ok(OneOrMany::from_iter(
            raw.into_iter().map(|x| x.into())
        ))
    }
}
impl TomlCode for Event {
    type RawFile = RawEventFile;
    fn from_raw_file(raw: <Self::RawFile as RawTomlCodeFile>::RawPieces) -> Result<OneOrMany<Self>, Error> {
        Ok(OneOrMany::from_iter(
            raw.into_iter().map(|x| x.into())
        ))
    }
}
impl TomlCode for EntityProto {
    type RawFile = RawEntityProtoFile;
    fn from_raw_file(raw: <Self::RawFile as RawTomlCodeFile>::RawPieces) -> Result<OneOrMany<Self>, Error> {
        let protos: Vec<_> = raw.into_iter().map(|x| x.try_into()).try_collect()?;
        //TODO: remove the try_collect
        Ok(OneOrMany::from_iter(
            protos.into_iter()
        ))
    }
}
impl TomlCode for System {
    type RawFile = RawSystemFile;
    fn from_raw_file(raw: <Self::RawFile as RawTomlCodeFile>::RawPieces) -> Result<OneOrMany<Self>, Error> {
        Ok(OneOrMany::One(Box::new(raw.into_pieces().try_into()?)))
    }
}


//Raw Transformation
impl From<RawComponentField> for ComponentField {
    fn from(value: RawComponentField) -> Self {
        ComponentField {
            name: value.name,
            type_spec: AethaumType::from_type_str(&value.type_spec),
            default_value: value.default,
            description: value.description,
        }
    }
}
impl From<RawComponent> for Component {
    fn from(value: RawComponent) -> Self {
        Component {
            name: value.name,
            fields: value.fields.map(|fields| fields.into_iter().map(|x| x.into()).collect()),
            description: value.description,
        }
    }
}
impl From<RawEventField> for EventField {
    fn from(value: RawEventField) -> Self {
        EventField {
            name: value.name,
            type_spec: AethaumType::from_type_str(&value.type_spec),
            description: value.description,
        }
    }
}
impl From<RawEvent> for Event {
    fn from(value: RawEvent) -> Self {
        Event {
            name: value.name,
            fields: value.fields.map(|fields| fields.into_iter().map(|x| x.into()).collect()),
            description: value.description,
        }
    }
}
impl TryFrom<RawEntityProto> for EntityProto {
    type Error = anyhow::Error;
    fn try_from(value: RawEntityProto) -> Result<Self, Self::Error> {
        Ok(EntityProto {
            name: value.name,
            description: value.description,
            components: value.components.into_iter().map(|x| x.try_into()).try_collect()?,
        })
    }
}
impl TryFrom<RawSystemQuery> for SystemQuery {
    type Error = anyhow::Error;
    fn try_from(value: RawSystemQuery) -> Result<Self, Self::Error> {
        Ok(SystemQuery {
            name: value.name,
            description: value.description,
            component_constraint: ComponentConstraint::try_from(
                (value.components_include, value.components_exclude)
            )?
        })
    }
}
impl TryFrom<RawSystemEventHandler> for SystemEventHandler {
    type Error = anyhow::Error; //TODO: better error type further
    fn try_from(value: RawSystemEventHandler) -> Result<Self, Self::Error> {
        Ok(SystemEventHandler {
            watch_for: value.watch_for.try_into()?,
            priority: match value.priority {
                Some(priority) => {
                   match priority {
                       toml::Value::Integer(i) => {
                           if i < 0 {
                               return Err(anyhow::anyhow!("Priority must be positive"));
                           }
                           i as u32
                       },
                       _ => return Err(anyhow::anyhow!("Priority must be an integer")),
                   }
                },
                None => 0
            },
            logic: LuaScript::from_embed_or_file(value.logic, value.logic_file.map(|x| PathBuf::from(x.as_str())))? // For Further Version, LuaScript type might not be SmartString
        })
    }
}
impl TryFrom<RawSystemUpdate> for SystemUpdate {
    type Error = anyhow::Error; //TODO: better error type further
    fn try_from(value: RawSystemUpdate) -> Result<Self, Self::Error> {
        Ok(SystemUpdate {
            interval: match value.interval {
                toml::Value::Integer(i) => {
                    if i <= 0 {
                        return Err(anyhow::anyhow!("Interval must be positive"));
                    }
                    Duration::from_secs(i as u64)
                }
                toml::Value::Float(f) => {
                    if f <= 0.0 {
                        return Err(anyhow::anyhow!("Interval must be positive"));
                    }
                    Duration::from_secs_f64(f)
                }
                _ => return Err(anyhow::anyhow!("Interval must be a number")),
            },
            condition: LuaScript::from_embed_or_file(value.condition, value.condition_file.map(|x| PathBuf::from(x.as_str())))?,
            logic: LuaScript::from_embed_or_file(value.logic, value.logic_file.map(|x| PathBuf::from(x.as_str())))?,
        })
    }
}
impl TryFrom<RawSystem> for System {
    type Error = anyhow::Error; //TODO: better error type further
    fn try_from(value: RawSystem) -> Result<Self, Self::Error> {
        Ok(System {
            normal: value.normal.into(),
            queries: value.queries.into_iter().map(|q| q.try_into()).try_collect()?,
            update: value.update.map(TryInto::try_into).transpose()?,
            event_handlers: value.event_handlers
                .into_iter()
                .map(|h| h.try_into())
                .try_collect()?,
        })
    }
}
//Ref Trait Register
impl AethaumRef for ComponentRef {
    fn to_global_ref(self, module_name: SmartString) -> Self {
        Self {
            name: self.name,
            module_name: Some(module_name)
        }
    }

    fn to_local_ref(self) -> Self {
        Self {
            name: self.name,
            module_name: None
        }
    }
}
impl AethaumRef for EventRef {
    fn to_global_ref(self, module_name: SmartString) -> Self {
        Self {
            name: self.name,
            module_name: Some(module_name)
        }
    }
    fn to_local_ref(self) -> Self {
        Self {
            name: self.name,
            module_name: None
        }
    }
}
impl AethaumRef for EntityProtoRef {
    fn to_global_ref(self, module_name: SmartString) -> Self {
        Self {
            name: self.name,
            module_name: Some(module_name)
        }
    }
    fn to_local_ref(self) -> Self {
        Self {
            name: self.name,
            module_name: None
        }
    }
}
impl AethaumRef for SystemRef {
    fn to_global_ref(self, module_name: SmartString) -> Self {
        Self {
            name: self.name,
            module_name: Some(module_name)
        }
    }
    fn to_local_ref(self) -> Self {
        Self {
            name: self.name,
            module_name: None
        }
    }
}
#[derive(Debug,PartialEq,Clone)]
pub struct WorldNormal {
    pub name: SmartString,
    pub version: SmartString,
    pub author:  SmartString
}
impl From<RawWorldNormal> for WorldNormal {
    fn from(value: RawWorldNormal) -> Self {
        WorldNormal {
            name: value.name,
            version: value.version,
            author: value.author
        }
    }
}
#[derive(Debug,PartialEq,Clone)]
pub struct WorldModules {
    pub modules: HashMap<SmartString, PathBuf>,
}
impl From<RawWorldModules> for WorldModules {
    fn from(value: RawWorldModules) -> Self {
        WorldModules {
            modules: value.modules.into_iter().map(|(k,v)| (k, PathBuf::from(v.as_str()))).collect()
        }
    }
}
#[derive(Debug,PartialEq,Clone)]
pub struct WorldBuild {
    pub output_dir: SmartString,
}
impl From<RawWorldBuild> for WorldBuild {
    fn from(value: RawWorldBuild) -> Self {
        WorldBuild {
            output_dir: value.output_dir,
        }
    }
}
#[derive(Debug,PartialEq,Clone)]
pub struct WorldCargo {

}
impl From<RawWorldCargo> for WorldCargo {
    fn from(_value: RawWorldCargo) -> Self {
        WorldCargo {}
    }
}
#[derive(Debug,PartialEq,Clone)]
pub struct World {
    pub normal: WorldNormal,
    pub modules: WorldModules,
    pub build: Option<WorldBuild>,
    pub cargo: Option<WorldCargo>,
}
impl From<RawWorld> for World {
    fn from(value: RawWorld) -> Self {
        World {
            normal: value.normal.into(),
            modules: value.modules.into(),
            build: value.build.map(Into::into),
            cargo: value.cargo.map(Into::into),
        }
    }
}
//Field Trait Implementation
impl Field for ComponentField {
    fn name_as_rust_ident(&self) -> Ident {
        Ident::new(&self.name, Span::call_site())
    }

    fn type_as_rust_ident(&self) -> Ident {
        self.type_spec.to_rust_type()
    }
}
impl Field for &ComponentField {
    fn name_as_rust_ident(&self) -> Ident {
        Ident::new(&self.name, Span::call_site())
    }

    fn type_as_rust_ident(&self) -> Ident {
        self.type_spec.to_rust_type()
    }
}
impl Field for EventField {
    fn name_as_rust_ident(&self) -> Ident {
        Ident::new(&self.name, Span::call_site())
    }

    fn type_as_rust_ident(&self) -> Ident {
        self.type_spec.to_rust_type()
    }
}
impl Field for &EventField {
    fn name_as_rust_ident(&self) -> Ident {
        Ident::new(&self.name, Span::call_site())
    }

    fn type_as_rust_ident(&self) -> Ident {
        self.type_spec.to_rust_type()
    }
}
//Describable trait
impl Describable for ComponentField {
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    fn field_description(&self) -> Option<impl Iterator<Item = (&str, &str)>> {
        self.description.as_ref().map(|description| std::iter::once((self.name.as_str(), description.as_str())))
    }
}
impl Describable for Component {
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    fn field_description(&self) -> Option<impl Iterator<Item=(&str, &str)>> {
        self.fields.as_ref().map(|fields| fields.iter()
            .filter_map(|field| {
                field.description.as_ref().map(|desc| (field.name.as_str(), desc.as_str()))
            }))
    }
}
impl Describable for EventField {
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    fn field_description(&self) -> Option<impl Iterator<Item=(&str, &str)>> {
        self.description.as_ref().map(|desc| std::iter::once((self.name.as_str(), desc.as_str())))
    }
}
impl Describable for Event {
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    fn field_description(&self) -> Option<impl Iterator<Item=(&str, &str)>> {
        self.fields.as_ref().map(|fields| fields.iter().filter_map(|field| {
            field.description.as_ref().map(|desc| (field.name.as_str(), desc.as_str()))
        }))
    }
}
impl Describable for EntityProto {
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    fn field_description(&self) -> Option<impl Iterator<Item=(&str, &str)>> {
        None::<std::iter::Empty<_>>
    }
}
impl Describable for SystemQuery {
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    fn field_description(&self) -> Option<impl Iterator<Item=(&str, &str)>> {
        None::<std::iter::Empty<_>>
    }
}
impl Describable for System {
    fn description(&self) -> Option<&str> {
        self.normal.description.as_deref()
    }
    fn field_description(&self) -> Option<impl Iterator<Item=(&str, &str)>> {
        None::<std::iter::Empty<_>>
    }
}