use crate::client;
use crate::handlers;
use crate::handlers::{Handler, Result};
use crate::models::GenericPost;
use regex::Regex;
use std::cell::RefCell;
use std::rc::Rc;

const MIN_NUM_PLAYERS: usize = 3;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
enum WWPlayerKind {
    Villager,
    Werewolf,
    Oracle,
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum WWPlayerStatus {
    Awake,
    Dead,
}

type WWPlayerId = String;

#[derive(Debug, Clone)]
struct WWPlayer {
    id: WWPlayerId,
    kind: WWPlayerKind,
    status: WWPlayerStatus,
    username: String,
}

#[derive(Eq, PartialEq, Clone, Debug)]
enum WWStepKind {
    Nightfall,
    WakeOracle,
    VoteOracle,
    WakeWerewolfs,
    VoteWerewolfs,
    NewDay,
    Vote,
}

pub struct WW<C> {
    client: Rc<C>,
    game: RefCell<WWGame>,
}

struct WWGame {
    pub game_owner: Option<WWPlayerId>,
    pub players: Vec<WWPlayer>,
    pub room_all: String,
    pub room_ww: String,
    pub started: bool,
    pub team_id: String,
    pub step: WWStepKind,
    pub last_dead: String,
}

impl WWGame {
    pub fn set_player_dead(&mut self, user_id: &str) {
        for p in self.players.iter_mut() {
            if p.id == user_id {
                p.status = WWPlayerStatus::Dead;
                self.last_dead = p.username.clone();
                break;
            }
        }
    }

    pub fn check_player(
        &self,
        user_id: &str,
        kind: Option<WWPlayerKind>,
        status: Option<WWPlayerStatus>,
    ) -> bool {
        for p in self.players.iter() {
            if p.id == user_id {
                let mut is = true;
                match kind {
                    Some(k) => {
                        is = is && (p.kind == k);
                    }
                    None => {}
                }
                match status {
                    Some(s) => {
                        is = is && (p.status == s);
                    }
                    None => {}
                };
                return is;
            }
        }
        false
    }

    pub fn game_has(&self, kind: WWPlayerKind) -> bool {
        for p in self.players.iter() {
            if p.kind == kind {
                return true;
            }
        }

        return false;
    }

    pub fn player_by_username(&self, username: &str) -> Option<WWPlayer> {
        for p in self.players.iter() {
            if p.username == username {
                return Some((*p).clone());
            }
        }

        None
    }
}

impl<C> WW<C> {
    pub fn new(client: Rc<C>) -> Self {
        WW {
            client: client,
            game: RefCell::new(WWGame {
                players: vec![],
                started: false,
                team_id: String::from(""),
                game_owner: None,
                room_ww: String::from(""),
                room_all: String::from(""),
                step: WWStepKind::Nightfall,
                last_dead: String::from(""),
            }),
        }
    }
}

impl<C> WW<C>
where
    C: client::Sender + client::Channel + client::Getter,
{
    fn post_all(&self, post: &GenericPost) -> handlers::Result {
        let mut post = post.clone();
        post.channel_id = self.game.borrow().room_all.clone();
        self.client.post(&post)?;

        Ok(())
    }

    fn post_ww(&self, post: &GenericPost) -> handlers::Result {
        let mut post = post.clone();
        post.channel_id = self.game.borrow().room_ww.clone();
        self.client.post(&post)?;

        Ok(())
    }

    fn post_kind(&self, kind: WWPlayerKind, post: GenericPost) -> handlers::Result {
        for p in self.game.borrow().players.iter() {
            if p.kind == kind {
                let mut d = post.clone();
                d.channel_id = p.id.clone();
                self.client.post(&d)?;
            }
        }

        Ok(())
    }

    fn post_oracle(&self, post: GenericPost) -> handlers::Result {
        return self.post_kind(WWPlayerKind::Oracle, post);
    }

    fn handle_game_started(&self, post: &GenericPost) -> handlers::Result {
        let msg = post.message.clone();
        let player_id = post.user_id.clone();
        let votereg = Regex::new(r"^!ww[\s]+vote[\s]+[@]?([\S]+).*$").unwrap();
        let mut loop_again = false;

        if msg.contains("ready") {
            self.game.borrow_mut().step = WWStepKind::Nightfall;
        }

        println!("DEBUG WW: step: {:?}", self.game.borrow().step);
        println!("DEBUG WW: players: {:?}", self.game.borrow().players);

        match self.game.borrow().step {
            WWStepKind::Nightfall => {
                self.post_all(&GenericPost::with_message("### La nuit tombe, les villageois s'endorment.\nTout est calme dans le village. Vous faites de beaux rêves et attendez d’être réveillés…"))?;
                self.game.borrow_mut().step = WWStepKind::WakeOracle;
                loop_again = true;
            }
            WWStepKind::WakeOracle => {
                loop_again = true;
                if !self.game.borrow().game_has(WWPlayerKind::Oracle) {
                    self.game.borrow_mut().step = WWStepKind::WakeWerewolfs;
                } else {
                    let mut usernames = vec![];
                    for p in self.game.borrow_mut().players.iter() {
                        if p.kind != WWPlayerKind::Oracle && p.status == WWPlayerStatus::Awake {
                            usernames.push(format!("@{}", p.username.clone()).clone());
                        }
                    }

                    let msg = format!("Vote avec `!ww vote username` pour voir la carte de `username`.\nChoisis parmi : {}", usernames.join(", "));

                    self.post_oracle(GenericPost::with_message(msg.as_str()))?;
                    self.game.borrow_mut().step = WWStepKind::VoteOracle;
                }
            }
            WWStepKind::VoteOracle => {
                for caps in votereg.captures_iter(msg.as_str()) {
                    let choosen = self
                        .game
                        .borrow()
                        .player_by_username(caps.get(1).unwrap().as_str());

                    // skip if player isn't the oracle
                    if !self.game.borrow().check_player(
                        player_id.as_str(),
                        Some(WWPlayerKind::Oracle),
                        Some(WWPlayerStatus::Awake),
                    ) {
                        return Ok(());
                    }

                    match choosen {
                        Some(p) => {
                            let msg = format!("{} est: {:?}\n\nMaintenant, tu peux te **rendormir, les autres vont jouer** :)", p.username, p.kind);
                            self.post_oracle(GenericPost::with_message(msg.as_str()))?;
                            self.game.borrow_mut().step = WWStepKind::WakeWerewolfs;
                            loop_again = true;
                        }
                        None => {
                            self.post_oracle(GenericPost::with_message(
                                "Utilisateur introuvable ou pas dans la partie",
                            ))?;
                        }
                    };
                }
            }
            WWStepKind::WakeWerewolfs => {
                let mut usernames = vec![];
                for p in self.game.borrow().players.iter() {
                    if p.kind != WWPlayerKind::Werewolf && p.status == WWPlayerStatus::Awake {
                        usernames.push(format!("@{}", p.username.clone()).clone());
                    }
                }
                let msg = format!(
                    "### Vous avez faim !

Choisissez un de ces villageoi: {}

**Attention !**

* Délibérez entre vous.
* Une fois d'accord, choisissez qui va voter. C'est une limitation technique, donc pas de chichi.
* Une fois fait, tapez `!ww vote username`
",
                    usernames.join(" | ")
                );
                self.post_ww(&GenericPost::with_message(msg.as_str()))?;
                self.game.borrow_mut().step = WWStepKind::VoteWerewolfs;
                loop_again = true;
            }
            WWStepKind::VoteWerewolfs => {
                for caps in votereg.captures_iter(msg.as_str()) {
                    let choosen = self
                        .game
                        .borrow()
                        .player_by_username(caps.get(1).unwrap().as_str());

                    if !self.game.borrow().check_player(
                        player_id.as_str(),
                        Some(WWPlayerKind::Werewolf),
                        Some(WWPlayerStatus::Awake),
                    ) {
                        return Ok(());
                    }

                    match choosen {
                        Some(p) => {
                            if p.kind == WWPlayerKind::Werewolf {
                                self.post_ww(&GenericPost::with_message("Je crois que vous avez un traitre parmi vous… les loups garous ne sont pas sensés s'entre tués :D"))?;
                            } else if p.status != WWPlayerStatus::Awake {
                                self.post_ww(&GenericPost::with_message("Il est déjà mort --'"))?;
                            } else {
                                let msg = format!("Très bien ! {} est mort !\n\nLa nuit va bientôt laisser place au jour, vous revepartez vous coucher…", p.username);
                                self.post_oracle(GenericPost::with_message(msg.as_str()))?;
                                self.game.borrow_mut().set_player_dead(p.id.as_str());
                                self.game.borrow_mut().step = WWStepKind::NewDay;
                                loop_again = true;
                            }
                        }
                        None => {
                            self.post_oracle(GenericPost::with_message(
                                "Utilisateur introuvable ou pas dans la partie",
                            ))?;
                        }
                    }
                }
            }
            WWStepKind::NewDay => {
                let mut usernames = vec![];

                for p in self.game.borrow().players.iter() {
                    if p.status == WWPlayerStatus::Awake {
                        usernames.push(format!("@{}", p.username));
                    }
                }

                let msg = format!(
                    "### Le jour se lève sur le village !

Malheureusement, @{} est mort !

Rendez-vous sur la place du village pour tenter de tirer tout ça au clair.

**Attention**

* **Concertez-vous avant de voter !**
* Le vote doit se faire par **une** personne, vivante : `!ww vote username`
* Les candidats possibles sont : {}


Choisissez convenablement !
",
                    self.game.borrow().last_dead,
                    usernames.join(" | ")
                );
                self.post_all(&GenericPost::with_message(msg.as_str()))?;
                self.game.borrow_mut().step = WWStepKind::Vote;
                loop_again = true;
            }
            WWStepKind::Vote => {
                if !self.game.borrow().check_player(
                    player_id.as_str(),
                    None,
                    Some(WWPlayerStatus::Awake),
                ) {
                    return Ok(());
                }
                for caps in votereg.captures_iter(msg.as_str()) {
                    let choosen = self
                        .game
                        .borrow()
                        .player_by_username(caps.get(1).unwrap().as_str());

                    match choosen {
                        Some(p) => {
                            self.game.borrow_mut().set_player_dead(p.id.as_str());
                            let msg = format!(
                                "@{} meurt d'une balle dans le dos… son corps s'écroule par terre.

Vous découvrez de qui il s'agissait : **{:?}**",
                                p.username, p.kind
                            );
                            self.post_all(&GenericPost::with_message(msg.as_str()))?;
                            self.game.borrow_mut().step = WWStepKind::Nightfall;
                            loop_again = true;
                        }
                        None => {}
                    };
                }
            }
        };

        if loop_again {
            // CHECK ENDGAME
            let mut has_ww = false;
            let mut has_normal = false;

            for p in self.game.borrow().players.iter() {
                if p.kind == WWPlayerKind::Werewolf && p.status != WWPlayerStatus::Dead {
                    has_ww = true;
                } else if p.status != WWPlayerStatus::Dead {
                    has_normal = true;
                }
            }

            if !has_ww || !has_normal {
                self.game.borrow_mut().started = false;
                self.game.borrow_mut().players.clear();
                self.post_all(&GenericPost::with_message("## Endgame !"))?;

                if !has_normal {
                    self.post_all(&GenericPost::with_message(
                        "Tous les villageois sont morts, les loups-garous ont gagnés ! Bravo !",
                    ))?;
                }

                if !has_ww {
                    self.post_all(&GenericPost::with_message(
                        "Tous les loups-garous sont morts, les villageois ont gagnés ! Bravo !",
                    ))?;
                }

                self.post_all(&GenericPost::with_message(
                    "Vous pouvez quitter les canaux et retrouver une activité normale :)",
                ))?;

                self.client
                    .archive_channel(self.game.borrow().room_all.as_str())?;
                self.client
                    .archive_channel(self.game.borrow().room_ww.as_str())?;

                return Ok(());
            }

            let mut post = post.clone();
            post.message = String::from("!ww loop_again"); // artificial command. only to be in conditions that can lead here.
            self.handle_game_started(&post)?;
        }

        Ok(())
    }

    fn handle_game_started_init(&self) -> handlers::Result {
        use rand::seq::SliceRandom;
        use rand::thread_rng;

        // ASSIGN PLAYER TYPES
        let mut rng = thread_rng();
        self.game.borrow_mut().players.shuffle(&mut rng);

        let mut all = vec![];
        let mut ww = vec![];

        let totp = self.game.borrow().players.len();

        // ASSIGN PLAYERS
        let mut pidx = 0;
        self.game.borrow_mut().players[pidx].kind = WWPlayerKind::Werewolf;

        if totp >= 5 {
            self.game.borrow_mut().players[pidx + 1].kind = WWPlayerKind::Werewolf;
        }

        if totp >= 8 {
            self.game.borrow_mut().players[pidx + 2].kind = WWPlayerKind::Werewolf;
            if self.game.borrow_mut().players.len() > 10 {
                pidx = pidx - 1;
            }
            self.game.borrow_mut().players[pidx + 3].kind = WWPlayerKind::Werewolf;
            self.game.borrow_mut().players[pidx + 4].kind = WWPlayerKind::Oracle;
        }

        for p in self.game.borrow().players.iter() {
            all.push(p.id.clone());
            if p.kind == WWPlayerKind::Werewolf {
                ww.push(p.id.clone());
            };
        }

        // CREATE ROOMS WITH PLAYERS
        let wwr_all =
            self.client
                .create_private(self.game.borrow().team_id.as_str(), "WW-VILLAGE", &all)?;
        self.game.borrow_mut().room_all = wwr_all.clone();

        let wwr_ww =
            self.client
                .create_private(self.game.borrow().team_id.as_str(), "WW-LOUPS", &ww)?;
        self.game.borrow_mut().room_ww = wwr_ww.clone();

        // SEND GREETINGS
        let greeting_text = format!("## Bienvenue au village !

Chers villageois et villageoises, voici les règles :\n

* {:?} loups-garous sont présents dans le village. Seuls les loups-garous peuvent communiquer entre eux et connaissent leur identité !
* Nous jouons avec la Voyante, le Chasseur et la Sorcière.
* Les règles et rôles peuvent être trouvés ici : https://ludos.brussels/ludo-luAPE/opac_css/doc_num.php?explnum_id=307
* Pour arrêter la partie : `!ww stop_game_now`. **Attention: il n'y a pas de demande de confirmation.**
* **Jouez le jeu !** : ne communiquez pas ici ni entre vous lorsque le village est endormis !
* À charge de la personne ayant créée la partie de confirmer que tous les joueurs et joueuses sont prêtes, puis démarrer en envoyant `!ww ready`
* Le bot ne mémorise pas tous vos choix : à vous de vous souvenir de qui vous avez regardé la carte par exemple !

Amusez-vous bien !
        ", ww.len());
        let mut gp_all = GenericPost::with_message(greeting_text.as_str());
        gp_all.channel_id = wwr_all;
        self.client.post(&gp_all)?;

        let mut gp_ww = GenericPost::with_message(
            "## Vous êtes les loups-garous !

Attention, nous vous trompez pas de canal quand vous communiquez :)

**Jouez le jeu !** : ne communiquez pas entre vous lorsque les loups-garous sont endormis !

Pour le reste, les mêmes règles s'appliquent, lisez le canal du village !",
        );
        gp_ww.channel_id = wwr_ww;
        self.client.post(&gp_ww)?;

        Ok(())
    }

    fn handle_no_game_started(&self, post: &GenericPost) -> handlers::Result {
        let player_id: WWPlayerId = post.user_id.clone();
        let message = post.message.as_str();
        let channel_id = post.channel_id.clone();
        let users = self.client.users_by_ids(vec![player_id.as_str()])?;

        if message.contains("start") {
            // STARTING A GAME
            if self.game.borrow().players.len() == 0 {
                self.game.borrow_mut().players.push(WWPlayer {
                    id: player_id.clone(),
                    kind: WWPlayerKind::Villager,
                    status: WWPlayerStatus::Awake,
                    username: users[0].username.clone(),
                });

                self.game.borrow_mut().team_id = post.team_id.clone();
                self.game.borrow_mut().game_owner = Some(player_id.clone());

                let msg = format!("**Une partie de loup-garou a démarré !**\nUtilise `!ww join` pour participer, il faut au minimum {} joueurs !", MIN_NUM_PLAYERS);
                self.client.reply(post, msg.as_str())?;
            } else if self.game.borrow().players.len() >= MIN_NUM_PLAYERS
                && !self.game.borrow().started
            {
                if self.game.borrow().game_owner.clone().unwrap() == player_id {
                    self.game.borrow_mut().started = true;
                    self.handle_game_started_init()?;
                    self.client.reply(post, "La partie commence ! Les joueurs sont invités à discuter dans les nouveaux cannaux créés pour l'occasion.")?;
                } else {
                    self.client.reply(
                        post,
                        "La partie ne peut être démarrée que par la personne l'ayant créée.",
                    )?;
                }
            } else {
                self.client
                    .reply(post, "Une partie est déjà en route, utilise `!ww join`.")?;
            }
        } else if message.contains("join") {
            // JOINING A GAME
            for p in self.game.borrow().players.iter() {
                if p.id == player_id {
                    self.client.reply(post, "Tu es déjà dans la partie !")?;
                    return Ok(());
                }
            }

            if self.game.borrow().players.len() < 1 {
                self.client
                    .reply(post, "Aucune partie n'a démarré. Utilise !ww start")?;
            } else {
                self.game.borrow_mut().players.push(WWPlayer {
                    id: player_id,
                    kind: WWPlayerKind::Villager,
                    status: WWPlayerStatus::Awake,
                    username: users[0].username.clone(),
                });
                if self.game.borrow().players.len() >= MIN_NUM_PLAYERS {
                    let mut post = GenericPost::with_message(
                        "La partie peut être lancée (créateur seulement) avec `!ww start`.",
                    );
                    post.channel_id = channel_id;
                    self.client.post(&post)?;
                } else {
                    let msg = format!(
                        "Plus que {} joueurs pour pouvoir lancer la partie !",
                        MIN_NUM_PLAYERS - self.game.borrow().players.len()
                    );
                    let mut post = GenericPost::with_message(msg.as_str());
                    post.channel_id = channel_id;
                    self.client.post(&post)?;
                }
            }
        }

        Ok(())
    }
}

impl<C> Handler for WW<C>
where
    C: client::Sender + client::Channel + client::Getter,
{
    type Data = GenericPost;

    fn name(&self) -> &str {
        "werewolf"
    }

    fn help(&self) -> Option<&str> {
        None
    }

    fn handle(&self, post: &GenericPost) -> Result {
        let message = post.message.as_str();

        if message.starts_with("!ww ") {
            if message.contains("stop_game_now") {
                self.game.borrow_mut().players.clear();
                self.game.borrow_mut().started = false;
                self.client.reply(post, "Jeu arrêté.")?;
            } else if !self.game.borrow().started {
                self.handle_no_game_started(post)?;
            } else {
                self.handle_game_started(post)?;
            }
        }

        Ok(())
    }
}
