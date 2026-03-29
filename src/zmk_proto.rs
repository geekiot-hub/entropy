/// Hand-written prost message structs for ZMK Studio protocol.
/// Based on https://github.com/zmkfirmware/zmk-studio-messages

// ─── Top-level envelope (studio.proto) ──────────────────────────────────────

#[derive(Clone, PartialEq, prost::Message)]
pub struct Request {
    #[prost(uint32, tag = "1")]
    pub request_id: u32,
    #[prost(oneof = "request::Subsystem", tags = "3, 4, 5")]
    pub subsystem: Option<request::Subsystem>,
}

pub mod request {
    #[derive(Clone, PartialEq, prost::Oneof)]
    pub enum Subsystem {
        #[prost(message, tag = "3")]
        Core(super::core::Request),
        #[prost(message, tag = "4")]
        Behaviors(super::behaviors::Request),
        #[prost(message, tag = "5")]
        Keymap(super::keymap::Request),
    }
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct Response {
    #[prost(oneof = "response::Type", tags = "1, 2")]
    pub r#type: Option<response::Type>,
}

pub mod response {
    #[derive(Clone, PartialEq, prost::Oneof)]
    pub enum Type {
        #[prost(message, tag = "1")]
        RequestResponse(super::RequestResponse),
        #[prost(message, tag = "2")]
        Notification(super::Notification),
    }
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct RequestResponse {
    #[prost(uint32, tag = "1")]
    pub request_id: u32,
    #[prost(oneof = "request_response::Subsystem", tags = "2, 3, 4, 5")]
    pub subsystem: Option<request_response::Subsystem>,
}

pub mod request_response {
    #[derive(Clone, PartialEq, prost::Oneof)]
    pub enum Subsystem {
        #[prost(message, tag = "2")]
        Meta(super::meta::Response),
        #[prost(message, tag = "3")]
        Core(super::core::Response),
        #[prost(message, tag = "4")]
        Behaviors(super::behaviors::Response),
        #[prost(message, tag = "5")]
        Keymap(super::keymap::Response),
    }
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct Notification {
    #[prost(oneof = "notification::Subsystem", tags = "2, 5")]
    pub subsystem: Option<notification::Subsystem>,
}

pub mod notification {
    #[derive(Clone, PartialEq, prost::Oneof)]
    pub enum Subsystem {
        #[prost(message, tag = "2")]
        Core(super::core::Notification),
        #[prost(message, tag = "5")]
        Keymap(super::keymap::Notification),
    }
}

// ─── meta.proto ─────────────────────────────────────────────────────────────

pub mod meta {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration)]
    #[repr(i32)]
    pub enum ErrorConditions {
        Generic = 0,
        UnlockRequired = 1,
        RpcNotFound = 2,
        MsgDecodeFailed = 3,
        MsgEncodeFailed = 4,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Response {
        #[prost(oneof = "response::ResponseType", tags = "1, 2")]
        pub response_type: Option<response::ResponseType>,
    }

    pub mod response {
        #[derive(Clone, PartialEq, prost::Oneof)]
        pub enum ResponseType {
            #[prost(bool, tag = "1")]
            NoResponse(bool),
            #[prost(enumeration = "super::ErrorConditions", tag = "2")]
            SimpleError(i32),
        }
    }
}

// ─── core.proto ─────────────────────────────────────────────────────────────

pub mod core {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration)]
    #[repr(i32)]
    pub enum LockState {
        Locked = 0,
        Unlocked = 1,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Request {
        #[prost(oneof = "request::RequestType", tags = "1, 2, 3, 4")]
        pub request_type: Option<request::RequestType>,
    }

    pub mod request {
        #[derive(Clone, PartialEq, prost::Oneof)]
        pub enum RequestType {
            #[prost(bool, tag = "1")]
            GetDeviceInfo(bool),
            #[prost(bool, tag = "2")]
            GetLockState(bool),
            #[prost(bool, tag = "3")]
            Lock(bool),
            #[prost(bool, tag = "4")]
            ResetSettings(bool),
        }
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Response {
        #[prost(oneof = "response::ResponseType", tags = "1, 2, 4")]
        pub response_type: Option<response::ResponseType>,
    }

    pub mod response {
        #[derive(Clone, PartialEq, prost::Oneof)]
        pub enum ResponseType {
            #[prost(message, tag = "1")]
            GetDeviceInfo(super::GetDeviceInfoResponse),
            #[prost(enumeration = "super::LockState", tag = "2")]
            GetLockState(i32),
            #[prost(bool, tag = "4")]
            ResetSettings(bool),
        }
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct GetDeviceInfoResponse {
        #[prost(string, tag = "1")]
        pub name: String,
        #[prost(bytes = "vec", tag = "2")]
        pub serial_number: Vec<u8>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Notification {
        #[prost(oneof = "notification::NotificationType", tags = "1")]
        pub notification_type: Option<notification::NotificationType>,
    }

    pub mod notification {
        #[derive(Clone, PartialEq, prost::Oneof)]
        pub enum NotificationType {
            #[prost(enumeration = "super::LockState", tag = "1")]
            LockStateChanged(i32),
        }
    }
}

// ─── behaviors.proto ────────────────────────────────────────────────────────

pub mod behaviors {
    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Request {
        #[prost(oneof = "request::RequestType", tags = "1, 2")]
        pub request_type: Option<request::RequestType>,
    }

    pub mod request {
        #[derive(Clone, PartialEq, prost::Oneof)]
        pub enum RequestType {
            #[prost(bool, tag = "1")]
            ListAllBehaviors(bool),
            #[prost(message, tag = "2")]
            GetBehaviorDetails(super::GetBehaviorDetailsRequest),
        }
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct GetBehaviorDetailsRequest {
        #[prost(uint32, tag = "1")]
        pub behavior_id: u32,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Response {
        #[prost(oneof = "response::ResponseType", tags = "1, 2")]
        pub response_type: Option<response::ResponseType>,
    }

    pub mod response {
        #[derive(Clone, PartialEq, prost::Oneof)]
        pub enum ResponseType {
            #[prost(message, tag = "1")]
            ListAllBehaviors(super::ListAllBehaviorsResponse),
            #[prost(message, tag = "2")]
            GetBehaviorDetails(super::GetBehaviorDetailsResponse),
        }
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct ListAllBehaviorsResponse {
        #[prost(uint32, repeated, tag = "1")]
        pub behaviors: Vec<u32>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct GetBehaviorDetailsResponse {
        #[prost(uint32, tag = "1")]
        pub id: u32,
        #[prost(string, tag = "2")]
        pub display_name: String,
        #[prost(message, repeated, tag = "3")]
        pub metadata: Vec<BehaviorBindingParametersSet>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct BehaviorBindingParametersSet {
        #[prost(message, repeated, tag = "1")]
        pub param1: Vec<BehaviorParameterValueDescription>,
        #[prost(message, repeated, tag = "2")]
        pub param2: Vec<BehaviorParameterValueDescription>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct BehaviorParameterValueDescription {
        #[prost(string, tag = "1")]
        pub name: String,
        #[prost(oneof = "behavior_parameter_value_description::ValueType", tags = "2, 3, 4, 5, 6")]
        pub value_type: Option<behavior_parameter_value_description::ValueType>,
    }

    pub mod behavior_parameter_value_description {
        #[derive(Clone, PartialEq, prost::Oneof)]
        pub enum ValueType {
            #[prost(message, tag = "2")]
            Nil(super::BehaviorParameterNil),
            #[prost(uint32, tag = "3")]
            Constant(u32),
            #[prost(message, tag = "4")]
            Range(super::BehaviorParameterValueDescriptionRange),
            #[prost(message, tag = "5")]
            HidUsage(super::BehaviorParameterHidUsage),
            #[prost(message, tag = "6")]
            LayerId(super::BehaviorParameterLayerId),
        }
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct BehaviorParameterNil {}

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct BehaviorParameterLayerId {}

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct BehaviorParameterHidUsage {
        #[prost(uint32, tag = "1")]
        pub keyboard_max: u32,
        #[prost(uint32, tag = "2")]
        pub consumer_max: u32,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct BehaviorParameterValueDescriptionRange {
        #[prost(int32, tag = "1")]
        pub min: i32,
        #[prost(int32, tag = "2")]
        pub max: i32,
    }
}

// ─── keymap.proto ───────────────────────────────────────────────────────────

pub mod keymap {
    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Request {
        #[prost(oneof = "request::RequestType", tags = "1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12")]
        pub request_type: Option<request::RequestType>,
    }

    pub mod request {
        #[derive(Clone, PartialEq, prost::Oneof)]
        pub enum RequestType {
            #[prost(bool, tag = "1")]
            GetKeymap(bool),
            #[prost(message, tag = "2")]
            SetLayerBinding(super::SetLayerBindingRequest),
            #[prost(bool, tag = "3")]
            CheckUnsavedChanges(bool),
            #[prost(bool, tag = "4")]
            SaveChanges(bool),
            #[prost(bool, tag = "5")]
            DiscardChanges(bool),
            #[prost(bool, tag = "6")]
            GetPhysicalLayouts(bool),
            #[prost(uint32, tag = "7")]
            SetActivePhysicalLayout(u32),
            #[prost(message, tag = "8")]
            MoveLayer(super::MoveLayerRequest),
            #[prost(message, tag = "9")]
            AddLayer(super::AddLayerRequest),
            #[prost(message, tag = "10")]
            RemoveLayer(super::RemoveLayerRequest),
            #[prost(message, tag = "11")]
            RestoreLayer(super::RestoreLayerRequest),
            #[prost(message, tag = "12")]
            SetLayerProps(super::SetLayerPropsRequest),
        }
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Response {
        #[prost(oneof = "response::ResponseType", tags = "1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12")]
        pub response_type: Option<response::ResponseType>,
    }

    pub mod response {
        #[derive(Clone, PartialEq, prost::Oneof)]
        pub enum ResponseType {
            #[prost(message, tag = "1")]
            GetKeymap(super::Keymap),
            #[prost(enumeration = "super::SetLayerBindingResponseCode", tag = "2")]
            SetLayerBinding(i32),
            #[prost(bool, tag = "3")]
            CheckUnsavedChanges(bool),
            #[prost(message, tag = "4")]
            SaveChanges(super::SaveChangesResponse),
            #[prost(bool, tag = "5")]
            DiscardChanges(bool),
            #[prost(message, tag = "6")]
            GetPhysicalLayouts(super::PhysicalLayouts),
            #[prost(message, tag = "7")]
            SetActivePhysicalLayout(super::SetActivePhysicalLayoutResponse),
            #[prost(message, tag = "8")]
            MoveLayer(super::MoveLayerResponse),
            #[prost(message, tag = "9")]
            AddLayer(super::AddLayerResponse),
            #[prost(message, tag = "10")]
            RemoveLayer(super::RemoveLayerResponse),
            #[prost(message, tag = "11")]
            RestoreLayer(super::RestoreLayerResponse),
            #[prost(enumeration = "super::SetLayerPropsResponseCode", tag = "12")]
            SetLayerProps(i32),
        }
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Notification {
        #[prost(oneof = "notification::NotificationType", tags = "1")]
        pub notification_type: Option<notification::NotificationType>,
    }

    pub mod notification {
        #[derive(Clone, PartialEq, prost::Oneof)]
        pub enum NotificationType {
            #[prost(bool, tag = "1")]
            UnsavedChangesStatusChanged(bool),
        }
    }

    // Enums used as oneof field types

    #[derive(Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration)]
    #[repr(i32)]
    pub enum SetLayerBindingResponseCode {
        Ok = 0,
        InvalidLocation = 1,
        InvalidBehavior = 2,
        InvalidParameters = 3,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration)]
    #[repr(i32)]
    pub enum SaveChangesErrorCode {
        Ok = 0,
        Generic = 1,
        NotSupported = 2,
        NoSpace = 3,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration)]
    #[repr(i32)]
    pub enum SetActivePhysicalLayoutErrorCode {
        Ok = 0,
        Generic = 1,
        InvalidLayoutIndex = 2,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration)]
    #[repr(i32)]
    pub enum MoveLayerErrorCode {
        Ok = 0,
        Generic = 1,
        InvalidLayer = 2,
        InvalidDestination = 3,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration)]
    #[repr(i32)]
    pub enum AddLayerErrorCode {
        Ok = 0,
        Generic = 1,
        NoSpace = 2,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration)]
    #[repr(i32)]
    pub enum RemoveLayerErrorCode {
        Ok = 0,
        Generic = 1,
        InvalidIndex = 2,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration)]
    #[repr(i32)]
    pub enum RestoreLayerErrorCode {
        Ok = 0,
        Generic = 1,
        InvalidId = 2,
        InvalidIndex = 3,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration)]
    #[repr(i32)]
    pub enum SetLayerPropsResponseCode {
        Ok = 0,
        Generic = 1,
        InvalidId = 2,
    }

    // Messages

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct SetLayerBindingRequest {
        #[prost(uint32, tag = "1")]
        pub layer_id: u32,
        #[prost(int32, tag = "2")]
        pub key_position: i32,
        #[prost(message, optional, tag = "3")]
        pub binding: Option<BehaviorBinding>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct MoveLayerRequest {
        #[prost(uint32, tag = "1")]
        pub start_index: u32,
        #[prost(uint32, tag = "2")]
        pub dest_index: u32,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct AddLayerRequest {}

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct RemoveLayerRequest {
        #[prost(uint32, tag = "1")]
        pub layer_index: u32,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct RestoreLayerRequest {
        #[prost(uint32, tag = "1")]
        pub layer_id: u32,
        #[prost(uint32, tag = "2")]
        pub at_index: u32,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct SetLayerPropsRequest {
        #[prost(uint32, tag = "1")]
        pub layer_id: u32,
        #[prost(string, tag = "2")]
        pub name: String,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct SaveChangesResponse {
        #[prost(oneof = "save_changes_response::Result", tags = "1, 2")]
        pub result: Option<save_changes_response::Result>,
    }

    pub mod save_changes_response {
        #[derive(Clone, PartialEq, prost::Oneof)]
        pub enum Result {
            #[prost(bool, tag = "1")]
            Ok(bool),
            #[prost(enumeration = "super::SaveChangesErrorCode", tag = "2")]
            Err(i32),
        }
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct SetActivePhysicalLayoutResponse {
        #[prost(oneof = "set_active_physical_layout_response::Result", tags = "1, 2")]
        pub result: Option<set_active_physical_layout_response::Result>,
    }

    pub mod set_active_physical_layout_response {
        #[derive(Clone, PartialEq, prost::Oneof)]
        pub enum Result {
            #[prost(message, tag = "1")]
            Ok(super::Keymap),
            #[prost(enumeration = "super::SetActivePhysicalLayoutErrorCode", tag = "2")]
            Err(i32),
        }
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct MoveLayerResponse {
        #[prost(oneof = "move_layer_response::Result", tags = "1, 2")]
        pub result: Option<move_layer_response::Result>,
    }

    pub mod move_layer_response {
        #[derive(Clone, PartialEq, prost::Oneof)]
        pub enum Result {
            #[prost(message, tag = "1")]
            Ok(super::Keymap),
            #[prost(enumeration = "super::MoveLayerErrorCode", tag = "2")]
            Err(i32),
        }
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct AddLayerResponse {
        #[prost(oneof = "add_layer_response::Result", tags = "1, 2")]
        pub result: Option<add_layer_response::Result>,
    }

    pub mod add_layer_response {
        #[derive(Clone, PartialEq, prost::Oneof)]
        pub enum Result {
            #[prost(message, tag = "1")]
            Ok(super::AddLayerResponseDetails),
            #[prost(enumeration = "super::AddLayerErrorCode", tag = "2")]
            Err(i32),
        }
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct AddLayerResponseDetails {
        #[prost(uint32, tag = "1")]
        pub index: u32,
        #[prost(message, optional, tag = "2")]
        pub layer: Option<Layer>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct RemoveLayerResponse {
        #[prost(oneof = "remove_layer_response::Result", tags = "1, 2")]
        pub result: Option<remove_layer_response::Result>,
    }

    pub mod remove_layer_response {
        #[derive(Clone, PartialEq, prost::Oneof)]
        pub enum Result {
            #[prost(message, tag = "1")]
            Ok(super::RemoveLayerOk),
            #[prost(enumeration = "super::RemoveLayerErrorCode", tag = "2")]
            Err(i32),
        }
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct RemoveLayerOk {}

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct RestoreLayerResponse {
        #[prost(oneof = "restore_layer_response::Result", tags = "1, 2")]
        pub result: Option<restore_layer_response::Result>,
    }

    pub mod restore_layer_response {
        #[derive(Clone, PartialEq, prost::Oneof)]
        pub enum Result {
            #[prost(message, tag = "1")]
            Ok(super::Layer),
            #[prost(enumeration = "super::RestoreLayerErrorCode", tag = "2")]
            Err(i32),
        }
    }

    // Core data types

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Keymap {
        #[prost(message, repeated, tag = "1")]
        pub layers: Vec<Layer>,
        #[prost(uint32, tag = "2")]
        pub available_layers: u32,
        #[prost(uint32, tag = "3")]
        pub max_layer_name_length: u32,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Layer {
        #[prost(uint32, tag = "1")]
        pub id: u32,
        #[prost(string, tag = "2")]
        pub name: String,
        #[prost(message, repeated, tag = "3")]
        pub bindings: Vec<BehaviorBinding>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct BehaviorBinding {
        #[prost(sint32, tag = "1")]
        pub behavior_id: i32,
        #[prost(uint32, tag = "2")]
        pub param1: u32,
        #[prost(uint32, tag = "3")]
        pub param2: u32,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct PhysicalLayouts {
        #[prost(uint32, tag = "1")]
        pub active_layout_index: u32,
        #[prost(message, repeated, tag = "2")]
        pub layouts: Vec<PhysicalLayout>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct PhysicalLayout {
        #[prost(string, tag = "1")]
        pub name: String,
        #[prost(message, repeated, tag = "2")]
        pub keys: Vec<KeyPhysicalAttrs>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct KeyPhysicalAttrs {
        #[prost(sint32, tag = "1")]
        pub width: i32,
        #[prost(sint32, tag = "2")]
        pub height: i32,
        #[prost(sint32, tag = "3")]
        pub x: i32,
        #[prost(sint32, tag = "4")]
        pub y: i32,
        #[prost(sint32, tag = "5")]
        pub r: i32,
        #[prost(sint32, tag = "6")]
        pub rx: i32,
        #[prost(sint32, tag = "7")]
        pub ry: i32,
    }
}
