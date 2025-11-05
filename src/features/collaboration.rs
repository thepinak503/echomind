use crate::error::{EchomindError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSession {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub owner: String,
    pub participants: Vec<Participant>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub is_active: bool,
    pub messages: Vec<CollaborationMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: String,
    pub name: String,
    pub role: ParticipantRole,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub is_online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipantRole {
    Owner,
    Editor,
    Viewer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationMessage {
    pub id: String,
    pub participant_id: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub message_type: MessageType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageType {
    Text,
    Code,
    File,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareableLink {
    pub id: String,
    pub conversation_id: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub password_protected: bool,
    pub allowed_views: u32,
    pub current_views: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct CollaborationManager {
    sessions: HashMap<String, CollaborationSession>,
    shareable_links: HashMap<String, ShareableLink>,
    current_user: String,
}

impl CollaborationManager {
    pub fn new(current_user: &str) -> Self {
        Self {
            sessions: HashMap::new(),
            shareable_links: HashMap::new(),
            current_user: current_user.to_string(),
        }
    }

    pub fn create_session(
        &mut self,
        name: &str,
        description: Option<&str>,
    ) -> Result<CollaborationSession> {
        let session = CollaborationSession {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            owner: self.current_user.clone(),
            participants: vec![Participant {
                id: self.current_user.clone(),
                name: self.current_user.clone(),
                role: ParticipantRole::Owner,
                joined_at: chrono::Utc::now(),
                is_online: true,
            }],
            created_at: chrono::Utc::now(),
            is_active: true,
            messages: Vec::new(),
        };
        
        self.sessions.insert(session.id.clone(), session.clone());
        Ok(session)
    }

    pub fn join_session(&mut self, session_id: &str, participant_name: &str) -> Result<CollaborationSession> {
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| EchomindError::Other(format!("Session {} not found", session_id)))?;
        
        let participant = Participant {
            id: Uuid::new_v4().to_string(),
            name: participant_name.to_string(),
            role: ParticipantRole::Editor,
            joined_at: chrono::Utc::now(),
            is_online: true,
        };
        
        session.participants.push(participant);
        
        let system_message = CollaborationMessage {
            id: Uuid::new_v4().to_string(),
            participant_id: "system".to_string(),
            content: format!("{} joined the session", participant_name),
            timestamp: chrono::Utc::now(),
            message_type: MessageType::System,
        };
        
        session.messages.push(system_message);
        Ok(session.clone())
    }

    pub fn leave_session(&mut self, session_id: &str, participant_id: &str) -> Result<()> {
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| EchomindError::Other(format!("Session {} not found", session_id)))?;
        
        if let Some(pos) = session.participants.iter().position(|p| p.id == participant_id) {
            let participant = &session.participants[pos];
            let system_message = CollaborationMessage {
                id: Uuid::new_v4().to_string(),
                participant_id: "system".to_string(),
                content: format!("{} left the session", participant.name),
                timestamp: chrono::Utc::now(),
                message_type: MessageType::System,
            };
            
            session.messages.push(system_message);
            session.participants.remove(pos);
        }
        
        Ok(())
    }

    pub fn send_message(
        &mut self,
        session_id: &str,
        participant_id: &str,
        content: &str,
        message_type: MessageType,
    ) -> Result<()> {
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| EchomindError::Other(format!("Session {} not found", session_id)))?;
        
        let message = CollaborationMessage {
            id: Uuid::new_v4().to_string(),
            participant_id: participant_id.to_string(),
            content: content.to_string(),
            timestamp: chrono::Utc::now(),
            message_type,
        };
        
        session.messages.push(message);
        Ok(())
    }

    pub fn get_session(&self, session_id: &str) -> Option<&CollaborationSession> {
        self.sessions.get(session_id)
    }

    pub fn list_sessions(&self) -> Vec<&CollaborationSession> {
        self.sessions.values().collect()
    }

    pub fn create_shareable_link(
        &mut self,
        conversation_id: &str,
        expires_in_hours: Option<u32>,
        password_protected: bool,
        max_views: Option<u32>,
    ) -> Result<ShareableLink> {
        let link = ShareableLink {
            id: Uuid::new_v4().to_string(),
            conversation_id: conversation_id.to_string(),
            expires_at: expires_in_hours.map(|h| chrono::Utc::now() + chrono::Duration::hours(h as i64)),
            password_protected,
            allowed_views: max_views.unwrap_or(100),
            current_views: 0,
            created_at: chrono::Utc::now(),
        };
        
        self.shareable_links.insert(link.id.clone(), link.clone());
        Ok(link)
    }

    pub fn get_shareable_content(&mut self, link_id: &str, password: Option<&str>) -> Result<String> {
        let conversation_id = {
            let link = self.shareable_links.get(link_id)
                .ok_or_else(|| EchomindError::Other("Shareable link not found".to_string()))?;

            // Check if link has expired
            if let Some(expires_at) = link.expires_at {
                if chrono::Utc::now() > expires_at {
                    return Err(EchomindError::Other("Shareable link has expired".to_string()));
                }
            }

            // Check view limit
            if link.current_views >= link.allowed_views {
                return Err(EchomindError::Other("Shareable link view limit exceeded".to_string()));
            }

            // Check password if required
            if link.password_protected && password.is_none() {
                return Err(EchomindError::Other("Password required for this link".to_string()));
            }

            link.conversation_id.clone()
        };

        // Increment view count
        if let Some(link) = self.shareable_links.get_mut(link_id) {
            link.current_views += 1;
        }

        // Return conversation content (placeholder)
        Ok(format!("Shared conversation content for: {}", conversation_id))
    }

    pub fn export_session(&self, session_id: &str, format: &str) -> Result<String> {
        let session = self.sessions.get(session_id)
            .ok_or_else(|| EchomindError::Other(format!("Session {} not found", session_id)))?;
        
        match format {
            "json" => {
                serde_json::to_string_pretty(session)
                    .map_err(|e| EchomindError::ParseError(format!("Failed to export session: {}", e)))
            }
            "markdown" => {
                let mut md = String::new();
                md.push_str(&format!("# {}\n\n", session.name));
                if let Some(description) = &session.description {
                    md.push_str(&format!("{}\n\n", description));
                }
                
                md.push_str("## Participants\n\n");
                for participant in &session.participants {
                    md.push_str(&format!("- **{}** ({})\n", participant.name, format!("{:?}", participant.role)));
                }
                
                md.push_str("\n## Messages\n\n");
                for message in &session.messages {
                    let participant_name = session.participants.iter()
                        .find(|p| p.id == message.participant_id)
                        .map(|p| p.name.as_str())
                        .unwrap_or("Unknown");
                    
                    md.push_str(&format!("**{}** - {}\n\n", participant_name, message.timestamp.format("%Y-%m-%d %H:%M:%S")));
                    md.push_str(&format!("{}\n\n", message.content));
                }
                
                Ok(md)
            }
            _ => Err(EchomindError::Other(format!("Unsupported export format: {}", format))),
        }
    }

    pub fn get_session_analytics(&self, session_id: &str) -> Result<SessionAnalytics> {
        let session = self.sessions.get(session_id)
            .ok_or_else(|| EchomindError::Other(format!("Session {} not found", session_id)))?;
        
        let total_messages = session.messages.len();
        let participant_count = session.participants.len();
        let duration = chrono::Utc::now() - session.created_at;
        
        let messages_by_participant: HashMap<String, usize> = session.messages.iter()
            .filter(|m| m.message_type != MessageType::System)
            .fold(HashMap::new(), |mut acc, msg| {
                *acc.entry(msg.participant_id.clone()).or_insert(0) += 1;
                acc
            });
        
        let most_active_participant = messages_by_participant.iter()
            .max_by_key(|(_, count)| *count)
            .map(|(id, count)| (id.clone(), *count));
        
        Ok(SessionAnalytics {
            session_id: session_id.to_string(),
            total_messages,
            participant_count,
            duration_hours: duration.num_hours() as f64,
            messages_by_participant,
            most_active_participant,
            is_active: session.is_active,
        })
    }

    pub fn end_session(&mut self, session_id: &str) -> Result<()> {
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| EchomindError::Other(format!("Session {} not found", session_id)))?;
        
        session.is_active = false;
        
        let system_message = CollaborationMessage {
            id: Uuid::new_v4().to_string(),
            participant_id: "system".to_string(),
            content: "Session ended".to_string(),
            timestamp: chrono::Utc::now(),
            message_type: MessageType::System,
        };
        
        session.messages.push(system_message);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAnalytics {
    pub session_id: String,
    pub total_messages: usize,
    pub participant_count: usize,
    pub duration_hours: f64,
    pub messages_by_participant: HashMap<String, usize>,
    pub most_active_participant: Option<(String, usize)>,
    pub is_active: bool,
}

impl Default for CollaborationManager {
    fn default() -> Self {
        Self::new("default_user")
    }
}