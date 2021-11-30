use std::sync::Arc;

use futures_locks::RwLock;
use serenity::{async_trait, builder::CreateApplicationCommands, client::Context, http::CacheHttp, model::{event::InteractionCreateEvent, id::{ApplicationId, GuildId, UserId}, interactions::application_command::{ApplicationCommand, ApplicationCommandInteraction, ApplicationCommandInteractionDataOption, ApplicationCommandInteractionDataOptionValue, ApplicationCommandOption, ApplicationCommandPermissionData, ApplicationCommandPermissionType}}};
use crate::component::{self as cmp, command_parser::{self as cmd, Named}, components::utils::{self, app_command::{ApplicationCommandEmbed, get_argument, unwrap_argument}}, manager::{ArcManager}};
use super::utils::message;
use crate::component::slash;

pub struct SlashInit {
    manager: ArcManager,
    owners: Vec<UserId>,
    group_match: cmd::Group,
    commands: RwLock<Vec<(GuildId, Vec<ApplicationCommand>)>>,
    app_id: ApplicationId,
}
#[async_trait]
impl cmp::Component for SlashInit {
    fn name(&self) -> &'static str {
        "slash"
    }

    async fn command(&self, fw_config: &cmp::FrameworkConfig, ctx: &cmp::Context, msg: &cmp::Message) -> cmp::CommandMatch {
        cmp::CommandMatch::NotMatched
    }

    async fn event(&self, ctx: &cmp::Context, evt: &cmp::Event) -> Result<(), String> {
        self.r_event(ctx, evt).await
    }
    fn group_parser(&self) -> Option<&cmd::Group> {
        Some(&self.group_match)
    }
}

macro_rules! slash_argument {
    
    ($app_cmd:ident, command: ($self: ident, $in_guild_id:ident, $out_opt_command: ident, $out_command_id:ident)) => {
        let $out_opt_command = match get_argument!($app_cmd, "command", String) {
            Some(v) => v,
            None => return message::error("L'identifiant de la commande est requis.")
        };
        let $out_command_id = {
            let commands = $self.commands.read().await;
            let (_, commands) = match commands.iter().find(|(g, _)| *g == $in_guild_id) {
                Some(list_commands) => list_commands,
                None => return message::error("Le serveur n'est pas reconnu.")
            };
            match commands.iter().find(|c| &c.name == $out_opt_command) {
                Some(command) => command.id,
                None => return message::error("Commande non trouvé.")
            }
        };
    };
    ($app_cmd:ident, who: $opt_name:ident) => {
        let $opt_name = match $app_cmd.get_argument("who") {
            Some(ApplicationCommandInteractionDataOption{
                resolved: Some(ApplicationCommandInteractionDataOptionValue::User(user, _)),
                ..
            }) => (user.id.0, ApplicationCommandPermissionType::User),
            Some(ApplicationCommandInteractionDataOption{
                resolved: Some(ApplicationCommandInteractionDataOptionValue::Role(role)),
                ..
            }) => (role.id.0, ApplicationCommandPermissionType::Role),
            None => return message::error("L'identifiant de l'utilisateur ou du rôle est requis."),
            _ => return message::error("L'identifiant de l'utilisateur ou du rôle n'est pas reconnu."),
        };
    };
    ($app_cmd:ident, type: $opt_name:ident) => {
        let $opt_name = match get_argument!($app_cmd, "type", String).and_then(|v| Some(v.as_str())) {
            Some("allow") => true,
            Some("deny") => false,
            Some(s) => return message::error(format!("Type: mot clé `{}` non reconnu. `allow` ou `deny` attendus.", s)),
            None => return message::error("Le type de permission est requis."), 
        };
    };
    ($app_cmd:ident, $($name:ident: $var_name:tt),+) => {
        $(
            slash_argument!($app_cmd, $name: $var_name);
        )*
    };
}

impl SlashInit {
    pub fn new(manager: ArcManager, owners: Vec<UserId>, app_id: ApplicationId) -> Self {
        use serenity::model::interactions::application_command::ApplicationCommandOptionType;
        let autocomplete_commands = Arc::new(Vec::new());
        let command = cmd::Command::new("")
            .set_help("Change le salon")
            .add_param(cmd::Argument::new("who")
                .set_value_type(ApplicationCommandOptionType::Mentionable)
                .set_required(true)
                .set_help("Qui est affecté")
            )
            .add_param(cmd::Argument::new("command")
                .set_value_type(ApplicationCommandOptionType::String)
                .set_required(true)
                .set_help("Quel commande est affecté")
                .set_autocomplete(autocomplete_commands.clone())
            )
            .add_param(cmd::Argument::new("type")
                .set_value_type(ApplicationCommandOptionType::String)
                .set_required(true)
                .set_help("Type d'autorisation")
                .set_autocomplete(Arc::new(vec![
                    "allow".to_string(),
                    "deny".to_string()
                ]))
            );
            
        let mut group_match = cmd::Group::new("slash")
            .set_help("Gestion des commandes slash")
            .set_permission("owners")
            .add_group(cmd::Group::new("permissions")
                .set_help("Gérer les permissions des commandes")
                .add_command({
                    let mut cmd = command.clone();
                    cmd.name = "set".into();
                    cmd
                })
                .add_command({
                    let mut cmd = command.clone();
                    cmd.name = "add".into();
                    cmd
                })
                .add_command(cmd::Command::new("list")
                    .set_help("Liste les permissions des commandes sur le serveur."))
            );
        group_match.generate_ids(None);
        SlashInit {
            commands: RwLock::new(Vec::new()),
            group_match,
            manager,
            owners,
            app_id
        }
    }
    async fn r_event(&self, ctx: &cmp::Context, evt: &cmp::Event) -> Result<(), String> {
        match evt {
            cmp::Event::Ready(ready) => {
                let manager = self.manager.read().await;
                let components = manager.get_components();
                let guilds = &ready.ready.guilds;
                let mut app_commands = CreateApplicationCommands::default();
                for compo in components {
                    let compo = compo.read().await;
                    let group = match compo.group_parser() {
                        Some(group) => group,
                        None => continue
                    };
                    app_commands.add_application_command(slash::register_root(group));
                }
                let mut commands = self.commands.write().await;
                for guild in guilds {
                    let guild_id = guild.id();
                    match guild_id.set_application_commands(ctx, |v| {
                        *v = app_commands.clone();
                        v
                    }).await {
                        Ok(v) => commands.push((guild_id, v)),
                        Err(why) => {
                            let name = guild.id().name(ctx).await.unwrap_or(guild.id().to_string());
                            eprintln!("Could not set application commands for guild {}: {:?}", name, why);
                        }
                    }
                }
                println!("Slash commands setted.");
            },
            cmp::Event::InteractionCreate(InteractionCreateEvent{interaction: serenity::model::interactions::Interaction::ApplicationCommand(c), ..}) => self.on_applications_command(ctx, c).await.unwrap_or(()),
            _ => (),
        }
        Ok(())
    }
    async fn on_applications_command(&self, ctx: &Context, app_cmd: &ApplicationCommandInteraction) -> Result<(), String> {
        if app_cmd.application_id != self.app_id {
            // La commande n'est pas destiné à ce bot
            return Ok(());
        }
        let app_command = ApplicationCommandEmbed::new(app_cmd);
        let guild_id = match app_command.get_guild_id() {
            Some(v) => v,
            None => return Err("Vous devez être dans un serveur pour utiliser cette commande.".into())
        };
        let command_name = app_command.fullname();
        let msg = match command_name.as_str() {
            "slash.permissions.add" => self.slash_perms_add(ctx, guild_id, app_command).await,
            "slash.permissions.list" => self.slash_perms_list(ctx, guild_id).await,
            _ => return Ok(())
        };
        app_cmd.create_interaction_response(ctx, |resp|{
            *resp = msg.into();
            resp
        }).await.or_else(|e| {
            eprintln!("Cannot create response: {}", e);
            Err(e.to_string())
        })
    }
    async fn slash_perms_add<'a>(&self, ctx: &Context, guild_id: GuildId, app_cmd: ApplicationCommandEmbed<'a>) -> message::Message {
        let user_id = app_cmd.0.member.as_ref().unwrap().user.id;
        if !self.owners.contains(&user_id) {
            return message::error("Cette commande est reservée aux owners");
        }

        let opt_command = match get_argument!(app_cmd, "command", String) {
            Some(opt_command) => opt_command,
            None => return message::error("L'identifiant de la commande est requis.")
        };
        let opt_type = match get_argument!(app_cmd, "type", String).and_then(|v| Some(v.as_str())) {
            Some("allow") => true,
            Some("deny") => false,
            Some(s) => return message::error(format!("Type: mot clé `{}` non reconnu. `allow` ou `deny` attendus.", s)),
            None => return message::error("Le type de permission est requis."), 
        };
        let opt_who = match app_cmd.get_argument("who") {
            Some(ApplicationCommandInteractionDataOption{
                resolved: Some(ApplicationCommandInteractionDataOptionValue::User(user, _)),
                ..
            }) => (user.id.0, ApplicationCommandPermissionType::User),
            Some(ApplicationCommandInteractionDataOption{
                resolved: Some(ApplicationCommandInteractionDataOptionValue::Role(role)),
                ..
            }) => (role.id.0, ApplicationCommandPermissionType::Role),
            None => return message::error("L'identifiant de l'utilisateur ou du rôle est requis."),
            _ => return message::error("L'identifiant de l'utilisateur ou du rôle n'est pas reconnu."),
        };
        let command_id = {
            let commands = self.commands.read().await;
            let (_, commands) = match commands.iter().find(|(g, _)| *g == guild_id) {
                Some(list_commands) => list_commands,
                None => return message::error("Le serveur n'est pas reconnu.")
            };
            match commands.iter().find(|c| &c.name == opt_command) {
                Some(command) => command.id,
                None => return message::error("Commande non trouvé.")
            }
        };
        let mut old_perms = match guild_id.get_application_command_permissions(ctx, command_id).await {
            Ok(v) => v.permissions,
            Err(_) => Vec::new()
        }.into_iter().map(|v| (v.id.0, v.kind, v.permission)).collect::<Vec<_>>();
        let updated = match old_perms.iter_mut().find(|v| v.0 == opt_who.0 && v.1 == opt_who.1) {
            Some(v) => {
                if v.2 == opt_type {
                    return message::success("La permission est déjà attribué tel quel.");
                }
                v.2 = opt_type;
                true
                
            },
            None => {
                old_perms.push((opt_who.0, opt_who.1, opt_type));
                false
            },
        };
        
        let result = guild_id.create_application_command_permission(ctx, command_id, |perm| {
            old_perms.iter().for_each(|p| {
                perm.create_permission(|new_perm| new_perm
                    .id(p.0)
                    .kind(p.1)
                    .permission(p.2)
                );
            });
            perm
        }).await;
        match (updated, result) {
            (true, Ok(_)) => message::success(format!("La permission de la commande `{}` a été mise a jour.", opt_command)),
            (false, Ok(_)) => message::success(format!("La permission de la commande `{}` a été ajoutée.", opt_command)),
            (_, Err(why)) => message::error(format!("La permission pour la commande {} n'a pas pu être assigné: {:?}", opt_command, why))
        }
    }
    async fn slash_perms_list<'a>(&self, ctx: &Context, guild_id: GuildId) -> message::Message {
        let commands = match guild_id.get_application_commands(ctx).await {
            Ok(v) => v,
            Err(_) => Vec::new()
        }.into_iter().filter(|c| c.application_id == self.app_id).collect::<Vec<_>>();
        let perms = match guild_id.get_application_commands_permissions(ctx).await {
            Ok(v) => v,
            Err(_) => Vec::new()
        }.into_iter().filter(|c| c.application_id == self.app_id).collect::<Vec<_>>();

        let perms = perms
            .into_iter()
            .filter_map(|v| {
                match commands.iter().find(|c| c.id == v.id) {
                    Some(command) => Some((command.name.clone(), v.permissions)),
                    None => None
                }
            })
            .map(|info_perms| {
                let list_perm = info_perms.1
                    .into_iter()
                    .map(|perm| {
                        let user = match perm.kind {
                            ApplicationCommandPermissionType::User => format!("<@{}>", perm.id),
                            ApplicationCommandPermissionType::Role => format!("<@&{}>", perm.id),
                            _ => "*unknown*".to_string(),
                        };
                        let permission = match perm.permission {
                            true => "est autorisé",
                            false => "est refusé",
                        };
                        format!("{} {}.\n", user, permission)
                    })
                    .collect::<String>();
                format!("*Commande __{}__*\n\n{}", info_perms.0, list_perm)
            })
            .collect::<String>();
        message::success(perms)
    }
}