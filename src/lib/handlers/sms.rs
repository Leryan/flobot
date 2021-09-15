use crate::client;
use crate::db;
use crate::handlers::{Error, Handler, Result};
use crate::models::GenericPost;
use regex::Regex;
use reqwest;
use serde_json::json;
use std::convert::From;
use std::rc::Rc;

pub enum SMSError {
    CannotSend(String),
}

impl From<SMSError> for Error {
    fn from(e: SMSError) -> Error {
        match e {
            SMSError::CannotSend(s) => Error::Other(s),
        }
    }
}

impl From<reqwest::Error> for SMSError {
    fn from(e: reqwest::Error) -> SMSError {
        SMSError::CannotSend(e.to_string())
    }
}

pub trait SMSSender {
    fn send(
        &self,
        text: &str,
        to_number: &str,
        from_name: &str,
    ) -> std::result::Result<(), SMSError>;
}

// FIXME: move implem elsewhere
pub struct Octopush {
    apikey: String,
    login: String,
}

impl Octopush {
    pub fn new(login: &str, apikey: &str) -> Self {
        Self {
            apikey: String::from(apikey),
            login: String::from(login),
        }
    }
}

impl SMSSender for Octopush {
    fn send(
        &self,
        text: &str,
        to_number: &str,
        from_name: &str,
    ) -> std::result::Result<(), SMSError> {
        let sms = json!({
            "sender": from_name,
            "recipients": [{"phone_number": to_number}],
            "text": text,
            "type": "sms_premium",
            "purpose": "alert",
            "with_replies": false,
        });
        let c = reqwest::blocking::Client::new();
        let r = c
            .post("https://api.octopush.com/v1/public/sms-campaign/send")
            .header("api-login", self.login.as_str())
            .header("api-key", self.apikey.as_str())
            .json(&sms)
            .send()?;

        match r.status() {
            reqwest::StatusCode::BAD_REQUEST => {
                println!("Sms Send 400: {:?}", r.text())
            }
            _ => println!("Sms Send: {:?}: {:?}", r.status(), r.text()),
        };

        Ok(())
    }
}

// END IMPLEM

pub struct SMS<S, D, C> {
    provider: S,
    db: Rc<D>,
    client: Rc<C>,
    re_register: Regex,
    re_prepare: Regex,
    re_send: Regex,
    re_sendn: Regex,
    re_list: Regex,
}

impl<S: SMSSender, D: db::SMS, C: client::Sender> SMS<S, D, C> {
    pub fn new(provider: S, db: Rc<D>, client: Rc<C>) -> Self {
        Self {
            db: db,
            provider: provider,
            client: client,
            re_register: Regex::new(r"^!sms[\s]+register[\s]+([a-zA-Z0-9\-_\.]+)[\s]+(\+[0-9]{11}).*$")
                .unwrap(),
            re_prepare: Regex::new(
                r"^!sms[\s]+prepare[\s]+([a-zA-Z0-9\-_\.]+)[\s]+([a-zA-Z0-9\-_\.]+)[\s]+([a-zA-Z0-9]+)[\s]+(.+)$",
            )
            .unwrap(),
            re_send: Regex::new(r"^!sms[\s]+([a-zA-Z0-9\-_\.]+)[\s]*$").unwrap(),
            re_list: Regex::new(r"^!sms[\s]+list[\s]*$").unwrap(),
            re_sendn: Regex::new(r"^!sms[\s]+send[\s]+([a-zA-Z0-9\-_\.]+)[\s]+([a-zA-Z0-9]+)[\s]+(.*)$").unwrap(),
        }
    }
}

impl<S: SMSSender, D: db::SMS, C: client::Sender> Handler for SMS<S, D, C> {
    type Data = GenericPost;

    fn name(&self) -> &str {
        "sms"
    }

    fn help(&self) -> Option<&str> {
        Some("Envoyer des sms.

 * **Pour un usage raisonné et responsable.** Oui c'est cliché mais le contenu des messages devient ma responsabilité :)
 * Le service a un coût, actuellement 21€ TTC pour 300 SMS à utiliser sur 1 an.
 * Le contenu des SMS n'est **pas** privé : je peux le voir dans la console de gestion. Ne vous en servez pas pour transmettre des infos privées.
 * Les numéros sont limités à la france et le préfixe `+33` est à mettre manuellement.

Voici comment procéder pour utiliser la fonctionnalité :

```
# Enregistrer un numéro associé à un nom/pseudo/ce que vous voulez.
!sms register <contact:[a-zA-Z0-9]+> <numéro>

# Préparer un envoi que vous pensez récurrent.
# Pour modifier un trigger, suffit de refaire la même commande mais avec un nom et/ou texte différent.
!sms prepare <trigname> <contact> <intitulé> <text>

# Faire un envoi préparé
!sms <trigname>

# Lister les contacts et trigs
!sms list
```

Pour chaque commande réussie, le bot ajoutera l'emoji :ok_hand: sur le message de commande.

Exemple :

```
!sms register paulin +33601020304
!sms send paulin Pwet Pwet !

!sms prepare paulin_bouffe paulin BOUFFE Ya dla bouffe, passe sur le chat !
!sms paulin_bouffe
```

**ATTENTION** :

 * Les espaces ne sont supportés QUE pour `<text>` et `<numéro>`. Pas de saut de ligne possible dans le texte.
 * L'envoi n'est pas garanti si le message contient des caractères exotiques, ça devrait quand même passer m'enfin attention.
 * Le service est payant, merci de ne pas en abuser :) On peut toujours s'arranger mais prévenez-moi avant :D

")
    }

    fn handle(&mut self, data: GenericPost) -> Result {
        let dc = data.clone();
        let msg = data.message.as_str();
        let tid = data.team_id.as_str();

        if !msg.starts_with("!sms ") {
            return Ok(());
        }

        if self.re_list.is_match(msg) {
            let mut msg = String::from("Contacts :\n\n");
            for c in self.db.list_contacts(tid)?.iter() {
                msg.push_str(format!("* {} -> `{}`\n", c.id, c.name).as_str());
            }
            msg.push_str("\nPréparations :\n\n");
            for p in self.db.list_prepare(tid)?.iter() {
                msg.push_str(
                    format!(
                        "{} -> contact n°{}, `!sms {}` enverra : {}: {}\n",
                        p.id, p.contact_id, p.trigname, p.name, p.text
                    )
                    .as_str(),
                );
            }
            self.client.reply(data, msg.as_str())?;
        } else if let Some(m) = self.re_send.captures(msg) {
            let trigname = m.get(1).unwrap();
            let prepare = self.db.get_prepare(tid, trigname.as_str())?;

            if let Some(prepare) = prepare {
                let contact = self.db.get_contact(tid, None, Some(&prepare.contact_id))?;
                if let Some(contact) = contact {
                    if let Err(e) = self.provider.send(
                        prepare.text.as_str(),
                        contact.number.as_str(),
                        prepare.name.as_str(),
                    ) {
                        self.client.reaction(data, "no_entry_sign")?;
                        return Err(e.into());
                    }
                }
            } else {
                self.client.reaction(data, ":question:")?;
            }
        } else if let Some(m) = self.re_sendn.captures(msg) {
            let contact_name = m.get(1).unwrap().as_str();
            let name = m.get(2).unwrap().as_str();
            let text = m.get(3).unwrap().as_str();

            if let Some(contact) = self.db.get_contact(tid, Some(contact_name), None)? {
                if let Err(e) = self.provider.send(text, contact.number.as_str(), name) {
                    self.client.reaction(data, "no_entry_sign")?;
                    return Err(e.into());
                }
            } else {
                let msg = format!("Pô trouvé {}", contact_name);
                self.client.reply(data, msg.as_str())?;
            }
        } else if let Some(m) = self.re_register.captures(msg) {
            let name = m.get(1).unwrap().as_str();
            let number = m.get(2).unwrap().as_str();
            self.db.set_contact(tid, name, number)?;
        } else if let Some(m) = self.re_prepare.captures(msg) {
            let trigname = m.get(1).unwrap().as_str();
            let contact_name = m.get(2).unwrap().as_str();
            let name = m.get(3).unwrap().as_str();
            let text = m.get(4).unwrap().as_str();

            if let Some(contact) = self.db.get_contact(tid, Some(contact_name), None)? {
                self.db
                    .set_prepare(tid, &contact.id, trigname, name, text)?;
            } else {
                let msg = format!("Pô trouvé {}", contact_name);
                self.client.reply(data, msg.as_str())?;
            }
        } else {
            self.client.reply(data, "L'a pô compris.")?;
            return Ok(());
        }

        self.client.reaction(dc, "ok_hand")?;

        Ok(())
    }
}
