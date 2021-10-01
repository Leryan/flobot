use rand::{prelude::SliceRandom, thread_rng};

#[derive(Clone, Debug, PartialEq)]
pub enum Role {
    Werewolf,
    Villager,
    Oracle,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub role: Role,
    pub alive: bool,
}

#[derive(PartialEq)]
pub enum Action {
    WaitPlayers,
    Ready,
    WhoWWKill,
    WWKill((String, String)), // player id votes to kill player name
    WhoVillageKill,
    VillageKill((String, String)), // player id votes to kill player name
    WhoDead,
}

#[derive(Debug)]
pub enum Error {
    IsDead,
    IsWerewolf,
    CannotDoNow,
    PlayerNotFound,
    NotEnoughPlayers,
}

#[derive(Debug, PartialEq)]
pub enum ActionAnswer {
    Ok,
    WaitPlayers,
    Ready,
    WhoWWKill(Vec<Player>),
    WWKill,
    WhoVillageKill(Vec<Player>),
    VillageKill(Player),
    WhoDead(Vec<Player>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Step {
    None,
    WaitPlayers,
    Ready,
    WerewolfsVoteKill,
    WerewolfsKill,
    NewDay,
    VillageVoteKill,
    VillageKill,
    End,
}

pub struct Game {
    players: Vec<Player>,
    step: Step,
    deads: Vec<Player>,
}

const MIN_PLAYERS: usize = 4;

impl Game {
    pub fn new() -> Self {
        Self {
            players: vec![],
            step: Step::None,
            deads: vec![],
        }
    }

    pub fn current_step(&self) -> Step {
        self.step.clone()
    }

    pub fn force_step(&mut self, action: Step) {
        self.step = action;
    }

    pub fn add_player(&mut self, id: &str, username: &str) -> Result<bool, Error> {
        if self.step != Step::WaitPlayers {
            return Err(Error::CannotDoNow);
        }

        if None == self.get_player(Some(id), Some(username)) {
            let player = Player {
                alive: true,
                role: Role::Villager,
                id: id.to_string(),
                name: username.to_string(),
            };
            self.players.push(player);
        }

        Ok(self.players.len() >= MIN_PLAYERS)
    }

    pub fn kill_player(
        &mut self,
        id: Option<&str>,
        name: Option<&str>,
    ) -> Option<Player> {
        for p in self.players.iter_mut() {
            if let Some(id) = id {
                if p.id == id {
                    p.alive = false;
                    return Some(p.clone());
                }
            } else if let Some(name) = name {
                if p.name == name {
                    p.alive = false;
                    return Some(p.clone());
                }
            }
        }
        None
    }

    pub fn all_players(&self) -> Vec<Player> {
        self.players.clone()
    }

    pub fn alive_players(&self) -> Vec<Player> {
        self.players
            .iter()
            .filter(|p| p.alive)
            .map(|p| p.clone())
            .collect()
    }

    pub fn alive_villagers(&self) -> Vec<Player> {
        self.players
            .iter()
            .filter(|p| p.alive && p.role != Role::Werewolf)
            .map(|p| p.clone())
            .collect()
    }

    pub fn alive_werewolfs(&self) -> Vec<Player> {
        self.players
            .iter()
            .filter(|p| p.alive && p.role == Role::Werewolf)
            .map(|p| p.clone())
            .collect()
    }

    /// get_player by id first, or name second.
    pub fn get_player(&self, id: Option<&str>, name: Option<&str>) -> Option<Player> {
        for p in self.players.iter() {
            if let Some(id) = id {
                if p.id == id {
                    return Some(p.clone());
                }
            }
            if let Some(name) = name {
                if p.name == name {
                    return Some(p.clone());
                }
            }
        }
        None
    }

    pub fn has_role(&self, role: Role) -> bool {
        for p in self.players.iter() {
            if p.role == role {
                return true;
            }
        }
        return false;
    }

    fn check_step_action(&self, action: &Action, step: &Step) -> Result<(), Error> {
        let res = match step {
            Step::None => *action == Action::WaitPlayers,
            Step::WaitPlayers => *action == Action::Ready,
            Step::Ready | Step::WerewolfsVoteKill => *action == Action::WhoWWKill,
            Step::WerewolfsKill => match action {
                Action::WWKill(_) => true,
                _ => false,
            },
            Step::NewDay => *action == Action::WhoDead,
            Step::VillageVoteKill => *action == Action::WhoVillageKill,
            Step::VillageKill => match action {
                Action::VillageKill(_) => true,
                _ => false,
            },
            Step::End => false,
        };

        if res {
            return Ok(());
        }

        Err(Error::CannotDoNow)
    }

    fn check_endgame_or(&self, step: Step) -> Step {
        if self.alive_werewolfs().len() == 0 || self.alive_villagers().len() == 0 {
            return Step::End;
        } else {
            step
        }
    }

    pub fn process(&mut self, action: Action) -> Result<ActionAnswer, Error> {
        self.check_step_action(&action, &self.step)?;

        match action {
            Action::WaitPlayers => {
                self.step = Step::WaitPlayers;
                Ok(ActionAnswer::Ok)
            }
            Action::Ready => {
                let mut rng = thread_rng();
                self.players.shuffle(&mut rng);
                let tot = self.players.len();
                if tot < MIN_PLAYERS {
                    return Err(Error::NotEnoughPlayers);
                }

                if tot <= MIN_PLAYERS + 2 {
                    self.players[0].role = Role::Werewolf;
                    self.players[1].role = Role::Werewolf;
                }

                if tot > MIN_PLAYERS + 2 {
                    self.players[2].role = Role::Werewolf;
                }

                self.step = Step::WerewolfsVoteKill;
                Ok(ActionAnswer::Ok)
            }
            Action::WhoWWKill => {
                self.step = Step::WerewolfsKill;
                Ok(ActionAnswer::WhoWWKill(self.alive_villagers()))
            }
            Action::WWKill((player, name)) => {
                if let Some(_) = self
                    .players
                    .iter()
                    .filter(|p| p.name == name && p.alive && p.role != Role::Werewolf)
                    .next()
                {
                    if self
                        .players
                        .iter()
                        .filter(|p| {
                            p.id == player && p.role == Role::Werewolf && p.alive
                        })
                        .count()
                        > 0
                    {
                        let player = self.kill_player(None, Some(&name)).unwrap();
                        self.deads.push(player);
                        self.step = self.check_endgame_or(Step::NewDay);
                        return Ok(ActionAnswer::WWKill);
                    } else {
                        return Err(Error::CannotDoNow);
                    }
                }
                return Err(Error::PlayerNotFound);
            }
            Action::WhoVillageKill => {
                self.step = Step::VillageKill;
                Ok(ActionAnswer::WhoVillageKill(self.alive_players()))
            }
            Action::VillageKill((player, name)) => {
                if let Some(_) = self
                    .players
                    .iter()
                    .filter(|p| p.name == name && p.alive)
                    .next()
                {
                    if self
                        .players
                        .iter()
                        .filter(|p| p.id == player && p.alive)
                        .count()
                        > 0
                    {
                        let player = self.kill_player(None, Some(&name)).unwrap();
                        self.deads.push(player.clone());
                        self.step = self.check_endgame_or(Step::WerewolfsVoteKill);
                        return Ok(ActionAnswer::VillageKill(player));
                    } else {
                        return Err(Error::CannotDoNow);
                    }
                }
                return Err(Error::PlayerNotFound);
            }
            Action::WhoDead => {
                let res = Ok(ActionAnswer::WhoDead(self.deads.clone()));
                self.deads.clear();
                self.step = self.check_endgame_or(Step::VillageVoteKill);
                res
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regular_scenario() -> Result<(), Error> {
        let mut game = Game::new();
        // try add player before moving to WaitPlayers
        let res = game.add_player("0", "zero");
        assert!(matches!(res, Err(Error::CannotDoNow)));

        let res = game.process(Action::WaitPlayers);
        assert!(res.is_ok());
        assert_eq!(Step::WaitPlayers, game.step);
        // add players. for as long as the game is not started, everything is cool.
        assert!(game.add_player("1", "one").is_ok());
        assert!(game.add_player("2", "two").is_ok());
        assert!(game.add_player("3", "three").is_ok());

        assert_eq!(game.step, Step::WaitPlayers);
        let res = game.process(Action::Ready);
        assert!(res.is_err());
        assert!(matches!(res, Err(Error::NotEnoughPlayers)));
        assert_eq!(game.step, Step::WaitPlayers);

        assert!(game.add_player("4", "four").is_ok());

        // start the game.
        let res = game.process(Action::Ready)?;
        assert_eq!(ActionAnswer::Ok, res);
        // asking werewolfs who they want to kill
        assert_eq!(game.step, Step::WerewolfsVoteKill);
        let res = game.process(Action::WhoWWKill)?;
        if let ActionAnswer::WhoWWKill(players) = res {
            assert_eq!(players.len(), 2);
            assert_eq!(
                players.iter().filter(|p| p.role != Role::Werewolf).count(),
                players.len()
            );
            let wwid = game.alive_werewolfs()[0].id.clone();

            let killp = players[0].clone();
            assert_eq!(true, killp.alive);
            let res = game.process(Action::WWKill((wwid, killp.name.clone())))?;

            assert_eq!(res, ActionAnswer::WWKill);
            assert_eq!(game.step, Step::NewDay);

            let res = game.process(Action::WhoDead)?;

            if let ActionAnswer::WhoDead(players) = res {
                assert_eq!(players.len(), 1);
                assert_eq!(players[0].id, killp.id);
                assert_eq!(players[0].name, killp.name);
                assert_eq!(false, players[0].alive);
            } else {
                assert!(false, "expected WhoDead");
            }
            assert_eq!(game.step, Step::VillageVoteKill);
            let res = game.process(Action::WhoVillageKill)?;
            if let ActionAnswer::WhoVillageKill(players) = res {
                assert_eq!(
                    players.iter().filter(|p| p.role == Role::Werewolf).count(),
                    2
                );
                assert_eq!(players.len(), 3);

                let villager = players
                    .iter()
                    .filter(|p| p.role == Role::Werewolf)
                    .next()
                    .unwrap();
                let viid = game.alive_players()[0].id.clone();
                let res =
                    game.process(Action::VillageKill((viid, villager.name.clone())))?;

                if let ActionAnswer::VillageKill(villager) = res {
                    assert_eq!(false, villager.alive);
                    assert_eq!(Role::Werewolf, villager.role);
                    assert_eq!(Step::WerewolfsVoteKill, game.step);

                    let res = game.process(Action::WhoWWKill)?;
                    if let ActionAnswer::WhoWWKill(players) = res {
                        assert_eq!(
                            players.iter().filter(|p| p.role != Role::Werewolf).count(),
                            1
                        );
                        assert_eq!(players.len(), 1);
                        assert_eq!(Step::WerewolfsKill, game.step);

                        let villager = players[0].clone();
                        let wwid = game.alive_werewolfs()[0].id.clone();
                        let res = game
                            .process(Action::WWKill((wwid, villager.name.clone())))?;

                        assert_eq!(ActionAnswer::WWKill, res);
                        assert_eq!(Step::End, game.step);
                    } else {
                        assert!(false, "expected WhoWWKill");
                    }
                } else {
                    assert!(false, "expected VillageKill");
                }
            } else {
                assert!(false, "expected WhoVillageKill");
            }
        } else {
            assert!(false, "expected WhoWWKill");
        }

        // ww vote

        Ok(())
    }
}
