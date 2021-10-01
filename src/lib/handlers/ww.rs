use crate::client;
use crate::handlers;
use crate::handlers::{Handler as BotHandler, Result};
use crate::models::Post;
use crate::werewolf;
use regex::Regex;
use std::cell::RefCell;
use std::convert::From;

pub struct Handler<C> {
    client: C,
    game: RefCell<werewolf::Game>,
    room_all: RefCell<String>,
    room_ww: RefCell<String>,
    team_id: RefCell<String>,
    game_owner: RefCell<Option<String>>,
}

impl<C> Handler<C> {
    pub fn new(client: C) -> Self {
        Handler {
            client: client,
            room_ww: RefCell::new(String::from("")),
            room_all: RefCell::new(String::from("")),
            team_id: RefCell::new(String::from("")),
            game_owner: RefCell::new(None),
            game: RefCell::new(werewolf::Game::new()),
        }
    }
}

const HELP: &'static str = "Le jeu se déroule en tour par tour.

 * La partie commence à la nuit tombante.
 * Les loups garous sortent et bouffent quelqu'un.
 * Le lendemain, le village trouve un cadavre.
 * Discussions, délibérations, accusations…
 * Vote ! Avec `!ww vote <name>`

Détails techniques :

 * Les loups garous choisissent leur proie sur leur canal avec `!ww vote <name>`.
 * Les villageois (loups garous cachés également !) parlent sur le canal `WW-VILLAGE`.
 * Quand la nuit tombe sur le village, seuls les loups garous peuvent parler.
 * Il n'est pas interdit de discuter en MP :D
 * Il est possible d'arrêter le jeu à n'importe quel moment avec `!ww stop_game_now`
 * Les votes utilisent toujours les *username* et se font comme suit : `!ww vote <username>`
";

impl<C> Handler<C>
where
    C: client::Sender + client::Channel + client::Getter,
{
    fn post_all(&self, post: &Post) -> handlers::Result {
        let post = post.nchannel(&self.room_all.borrow());
        self.client.post(&post)?;
        Ok(())
    }

    fn post_ww(&self, post: &Post) -> handlers::Result {
        let post = post.nchannel(&self.room_ww.borrow());
        self.client.post(&post)?;
        Ok(())
    }

    fn re_match(&self, re: &str, txt: &str) -> bool {
        Regex::new(re).unwrap().is_match(txt)
    }

    fn reset_game(&self) {
        *self.game_owner.borrow_mut() = None;
        *self.game.borrow_mut() = werewolf::Game::new();
    }

    fn handle_starting_commands(
        &self,
        post: &Post,
        cur: &werewolf::Step,
    ) -> handlers::Result {
        // answer to start, join and list commands
        if self.re_match(r"!ww[\s]+start.*", post.message.as_str()) {
            match cur {
                werewolf::Step::None => {
                    *self.team_id.borrow_mut() = post.team_id.clone();
                    *self.game_owner.borrow_mut() = Some(post.user_id.clone());
                    if self
                        .game
                        .borrow_mut()
                        .process(werewolf::Action::WaitPlayers)
                        .is_ok()
                    {
                        let users =
                            self.client.users_by_ids(vec![post.user_id.as_str()])?;
                        if self
                            .game
                            .borrow_mut()
                            .add_player(
                                users[0].id.as_str(),
                                users[0].username.as_str(),
                            )
                            .is_ok()
                        {
                            self.client.reaction(&post, "ok_hand")?;
                            self.client.post(&post.nmessage(
                                "Une partie de loup-garou va démarrer ! Pour joindre la partie : `!ww join`",
                            ))?;
                        }
                    }
                }
                werewolf::Step::WaitPlayers => {
                    if let Some(go) = self.game_owner.borrow().clone() {
                        if go == post.user_id.as_str() {
                            let res =
                                self.game.borrow_mut().process(werewolf::Action::Ready);
                            if let Ok(_) = res {
                                let all: Vec<String> = self
                                    .game
                                    .borrow()
                                    .alive_players()
                                    .iter()
                                    .filter_map(|p| Some(p.id.clone()))
                                    .collect();
                                let ww: Vec<String> = self
                                    .game
                                    .borrow()
                                    .alive_werewolfs()
                                    .iter()
                                    .filter_map(|p| Some(p.id.clone()))
                                    .collect();

                                let rid_all = self.client.create_private(
                                    self.team_id.borrow().as_str(),
                                    "WW-VILLAGE",
                                    &all,
                                )?;
                                let rid_ww = self.client.create_private(
                                    self.team_id.borrow().as_str(),
                                    "WW-LOUPS",
                                    &ww,
                                )?;
                                *self.room_all.borrow_mut() = rid_all;
                                *self.room_ww.borrow_mut() = rid_ww;
                                self.post_all(
                                    &post.nmessage("### La partie commence !"),
                                )?;
                                self.post_all(&post.nmessage(HELP))?;
                            }
                        }
                    }
                }
                _ => self.client.reply(post, "Une partie est déjà en cours.")?,
            };
        } else if self.re_match(r"!ww[\s]+join.*", post.message.as_str()) {
            match cur {
                werewolf::Step::WaitPlayers => {
                    let users =
                        self.client.users_by_ids(vec![post.user_id.as_str()])?;
                    let res = self
                        .game
                        .borrow_mut()
                        .add_player(users[0].id.as_str(), users[0].username.as_str());
                    if res.is_ok() {
                        self.client.reaction(&post, "ok_hand")?;
                        if res.unwrap() {
                            self.client.post(&post.nmessage("La partie peut démarrer. Il est toujours possible de joindre la partie. Quand vous êtes prêts, démarrez avec `!ww start`"))?;
                        }
                    }
                }
                _ => self
                    .client
                    .reply(post, "Aucune partie joignable pour le moment.")?,
            };
        } else if self.re_match(r"!ww[\s]+list.*", post.message.as_str()) {
            match cur {
                werewolf::Step::WaitPlayers => {
                    let mut msg = String::from("Joueurs en attente : ");
                    for p in self.game.borrow().all_players().iter() {
                        msg.push_str(format!("{} ", p.name).as_str());
                    }
                    self.client.reply(post, msg.as_str())?;
                }
                _ => self.client.reply(post, "Aucune partie en attente.")?,
            };
        };

        Ok(())
    }

    fn handle_game(&self, post: &Post) -> handlers::Result {
        let cur = self.game.borrow().current_step();
        self.handle_starting_commands(post, &cur)?;

        let re_vote = Regex::new(r"!ww[\s]+vote[\s]+([\S]+)[\s]*").unwrap();

        loop {
            println!("WW GAME STEP: {:?}", self.game.borrow().current_step());
            let step = self.game.borrow().current_step();
            match step {
                werewolf::Step::None | werewolf::Step::Ready => {}
                werewolf::Step::WaitPlayers => break,
                werewolf::Step::WerewolfsVoteKill => {
                    let res =
                        self.game.borrow_mut().process(werewolf::Action::WhoWWKill);
                    if let Ok(werewolf::ActionAnswer::WhoWWKill(players)) = res {
                        let names = players
                            .iter()
                            .map(|p| format!(" * `{}`", p.name))
                            .collect::<Vec<String>>()
                            .join("\n");
                        self.post_all(&post.nmessage(
                            "### Le soleil se couche, les villageois aussi…",
                        ))?;
                        let msg = format!("### Vous avez FAIM !\nChoisissez avec `!ww vote <name>` :\n{}", names);
                        self.post_ww(&post.nmessage(msg.as_str()))?;
                        break;
                    }
                }
                werewolf::Step::WerewolfsKill => {
                    if let Some(captures) = re_vote.captures(post.message.as_str()) {
                        let name = captures.get(1).unwrap().as_str().to_string();
                        let res =
                            self.game.borrow_mut().process(werewolf::Action::WWKill((
                                post.user_id.clone(),
                                name.clone(),
                            )));
                        if let Ok(werewolf::ActionAnswer::WWKill) = res {
                            let msg = format!("{} était bien bon…", name);
                            self.post_ww(&post.nmessage(msg.as_str()))?;
                        } else {
                            self.client.reply(&post, "pas possible")?;
                            break;
                        }
                    }
                }
                werewolf::Step::NewDay => {
                    let res = self.game.borrow_mut().process(werewolf::Action::WhoDead);
                    if let Ok(werewolf::ActionAnswer::WhoDead(players)) = res {
                        let names = players
                            .iter()
                            .map(|p| format!(" * `{}` était {:?}", p.name, p.role))
                            .collect::<Vec<String>>()
                            .join("\n");
                        let msg = format!("### Quelqu'un est mort…\n{}", names);
                        self.post_all(&post.nmessage(msg.as_str()))?;
                    }
                }
                werewolf::Step::VillageVoteKill => {
                    let res = self
                        .game
                        .borrow_mut()
                        .process(werewolf::Action::WhoVillageKill);
                    if let Ok(werewolf::ActionAnswer::WhoVillageKill(players)) = res {
                        let names = players
                            .iter()
                            .map(|p| format!(" * `{}`", p.name))
                            .collect::<Vec<String>>()
                            .join("\n");
                        let msg = format!(
                            "### Votez qui selon vous est un loup garou !\n{}",
                            names
                        );
                        self.post_all(&post.nmessage(msg.as_str()))?;
                        break;
                    }
                }
                werewolf::Step::VillageKill => {
                    if let Some(captures) = re_vote.captures(post.message.as_str()) {
                        let name = captures.get(1).unwrap().as_str().to_string();
                        let res = self.game.borrow_mut().process(
                            werewolf::Action::VillageKill((
                                post.user_id.clone(),
                                name.clone(),
                            )),
                        );
                        if let Ok(werewolf::ActionAnswer::VillageKill(player)) = res {
                            let msg = format!("`{}` était {:?} !", name, player.role);
                            self.post_all(&post.nmessage(msg.as_str()))?;
                        } else {
                            self.client.reply(&post, "pas possible")?;
                            break;
                        }
                    }
                }
                werewolf::Step::End => {
                    let mut msg = String::from("### Fin de partie !\n\n");
                    if self.game.borrow().alive_villagers().len() == 0 {
                        msg.push_str("Les loups-garous ont gagnés !");
                    } else {
                        msg.push_str("Les villageois ont gagnés !");
                    }
                    msg.push_str("\n\n**Pensez à archiver le canal :)**");

                    self.reset_game();

                    let _ = self.client.archive_channel(self.room_ww.borrow().as_str());

                    self.post_all(&post.nmessage(msg.as_str()))?;
                    break;
                }
            };
        }

        Ok(())
    }
}

impl<C> BotHandler for Handler<C>
where
    C: client::Sender + client::Channel + client::Getter,
{
    type Data = Post;

    fn name(&self) -> &str {
        "werewolf"
    }

    fn help(&self) -> Option<String> {
        Some(
            "### Jeu du loup garou

 * Commencer une partie avec `!ww start`
 * Joindre une partie annoncée avec `!ww join`
 * Quand le bot annonce que la partie peut être lancée, suivre les instructions :)
"
            .to_string(),
        )
    }

    fn handle(&self, post: &Post) -> Result {
        let message = post.message.as_str();

        if !message.starts_with("!ww ") {
            return Ok(());
        }

        if message.starts_with("!ww stop_game_now") {
            self.reset_game();
            self.client.reply(post, "Jeu arrêté.")?;
        } else {
            self.handle_game(post)?;
        }

        Ok(())
    }
}
