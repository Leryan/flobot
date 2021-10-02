use crate::db::models::{NewSMSContact, NewSMSPrepare};
use crate::db::schema::sms_contact::dsl as tc;
use crate::db::schema::sms_prepare::dsl as tp;
use crate::db::Error;
use crate::db::Result;
use crate::models;
use diesel::prelude::*;

impl crate::db::SMS for super::Sqlite {
    fn set_contact(
        &self,
        team_id: &str,
        name: &str,
        number: &str,
    ) -> Result<models::SMSContact> {
        let db = &*self.db.lock().unwrap();
        db.transaction::<(), Error, _>(|| {
            let res = tc::sms_contact
                .filter(tc::team_id.eq(team_id).and(tc::name.eq(name)))
                .first::<models::SMSContact>(db);

            match res {
                Ok(contact) => {
                    diesel::update(tc::sms_contact.filter(tc::id.eq(contact.id)))
                        .set(tc::number.eq(number))
                        .execute(db)?;
                    Ok(())
                }
                Err(diesel::NotFound) => {
                    let contact = NewSMSContact {
                        team_id: team_id,
                        last_sending_unixts: &0,
                        name: name,
                        number: number,
                    };
                    diesel::insert_into(tc::sms_contact)
                        .values(&contact)
                        .execute(db)?;
                    Ok(())
                }
                Err(e) => Err(e),
            }?;

            Ok(())
        })?;

        Ok(tc::sms_contact
            .filter(
                tc::name
                    .eq(name)
                    .and(tc::number.eq(number))
                    .and(tc::team_id.eq(team_id)),
            )
            .first(db)?)
    }

    fn list_contacts(&self, team_id: &str) -> Result<Vec<models::SMSContact>> {
        Ok(tc::sms_contact
            .filter(tc::team_id.eq(team_id))
            .order_by(tc::name)
            .load(&*self.db.lock().unwrap())?)
    }

    fn set_prepare(
        &self,
        team_id: &str,
        contact_id: &i32,
        trigname: &str,
        name: &str,
        text: &str,
    ) -> Result<models::SMSPrepare> {
        let db = &*self.db.lock().unwrap();
        db.transaction::<_, Error, _>(|| {
            let res = tp::sms_prepare
                .filter(
                    tp::team_id
                        .eq(team_id)
                        .and(tp::sms_contact_id.eq(contact_id))
                        .and(tp::trigname.eq(trigname)),
                )
                .first::<models::SMSPrepare>(db);

            match res {
                Ok(prepare) => {
                    diesel::update(tp::sms_prepare.filter(tp::id.eq(prepare.id)))
                        .set((tp::name.eq(name), tp::text.eq(text)))
                        .execute(db)?;
                    Ok(())
                }
                Err(diesel::NotFound) => {
                    let prepare = NewSMSPrepare {
                        team_id: team_id,
                        sms_contact_id: contact_id,
                        name: name,
                        text: text,
                        trigname: trigname,
                    };
                    diesel::insert_into(tp::sms_prepare)
                        .values(&prepare)
                        .execute(db)?;
                    Ok(())
                }
                Err(e) => Err(e),
            }?;

            Ok(())
        })?;

        Ok(tp::sms_prepare
            .filter(
                tp::team_id
                    .eq(team_id)
                    .and(tp::sms_contact_id.eq(contact_id))
                    .and(tp::trigname.eq(trigname)),
            )
            .first(db)?)
    }

    fn list_prepare(
        &self,
        team_id: &str,
    ) -> Result<Vec<(models::SMSPrepare, models::SMSContact)>> {
        let res = tp::sms_prepare
            .filter(tp::team_id.eq(team_id))
            .order_by(tp::trigname)
            .inner_join(tc::sms_contact)
            .load(&*self.db.lock().unwrap())?;
        Ok(res)
    }

    fn get_contact(
        &self,
        team_id: &str,
        name: Option<&str>,
        id: Option<&i32>,
    ) -> Result<Option<models::SMSContact>> {
        let mut query = tc::sms_contact.into_boxed();
        query = query.filter(tc::team_id.eq(team_id));
        if let Some(name) = name {
            query = query.filter(tc::name.eq(name));
        }
        if let Some(id) = id {
            query = query.filter(tc::id.eq(id));
        }
        match query.first(&*self.db.lock().unwrap()) {
            Ok(contact) => Ok(Some(contact)),
            Err(diesel::NotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn get_prepare(
        &self,
        team_id: &str,
        trigname: &str,
    ) -> Result<Option<models::SMSPrepare>> {
        match tp::sms_prepare
            .filter(tp::team_id.eq(team_id).and(tp::trigname.eq(trigname)))
            .first(&*self.db.lock().unwrap())
        {
            Ok(prepare) => Ok(Some(prepare)),
            Err(diesel::NotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
