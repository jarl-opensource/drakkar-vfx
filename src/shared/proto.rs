use bevy_hanabi::EffectAsset;
use serde::{Deserialize, Serialize};

/// Messages that can be sent from external processes to the server
#[derive(Clone, Serialize, Deserialize)]
pub enum ServerCommandMessage
{
    Ping,
    OpenAsset
    {
        asset: EffectAsset,
    },
    OpenAssetFile
    {
        path: String,
    },
}

/// Responses sent back to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerCommandResponse
{
    Ok,
    Error
    {
        message: String,
    },
}
