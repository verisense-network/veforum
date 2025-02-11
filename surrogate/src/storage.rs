use parity_scale_codec::Encode;
use rocksdb::{Options, WriteBatchWithTransaction, DB};
use vemodel::*;

const EVENT_PREFIX: u128 = 0xffffffff_ffffffff_00000000_00000000;

pub fn open(path: impl AsRef<std::path::Path>) -> anyhow::Result<DB> {
    let mut opts = Options::default();
    opts.create_if_missing(true);
    DB::open(&opts, path).map_err(Into::into)
}

pub fn save_community(db: &DB, community: &Community) -> anyhow::Result<()> {
    db.put(community.id.to_be_bytes(), &community.encode())?;
    Ok(())
}

// pub fn save_events(db: &DB, events: Vec<(EventId, Event)>) -> anyhow::Result<(EventId, EventId)> {
//     let start = events.first().map(|(id, _)| *id).unwrap();
//     let end = events.last().map(|(id, _)| *id).unwrap();
//     let mut batch = WriteBatchWithTransaction::<false>::default();
//     for (id, event) in events {
//         let key = EVENT_PREFIX | id as u128;
//         batch.put(key.to_be_bytes(), &event.encode());
//     }
//     db.write(batch)?;
//     Ok((start, end))
// }

pub fn save_event(db: &DB, event_id: EventId, event: Event) -> anyhow::Result<()> {
    let key = EVENT_PREFIX | event_id as u128;
    db.put(key.to_be_bytes(), &event.encode())?;
    Ok(())
}

pub fn get_max_event(db: &DB) -> anyhow::Result<EventId> {
    db.iterator(rocksdb::IteratorMode::End)
        .next()
        .transpose()?
        .filter(|(k, _)| k.starts_with(&EVENT_PREFIX.to_be_bytes()[..]))
        .map(|(key, _)| {
            let id = u128::from_be_bytes((*key).try_into().expect("Invalid event id"));
            Ok(id as EventId)
        })
        .unwrap_or(Ok(1))
}

pub fn save_contents(db: &DB, contents: &[(ContentId, Vec<u8>)]) -> anyhow::Result<()> {
    let mut batch = WriteBatchWithTransaction::<false>::default();
    for (id, content) in contents {
        batch.put(id.to_be_bytes(), &content);
    }
    db.write(batch)?;
    Ok(())
}

pub fn del_content(db: &DB, id: ContentId) -> anyhow::Result<()> {
    db.delete(id.to_be_bytes())?;
    Ok(())
}

pub fn exists(db: &DB, id: impl AsRef<[u8]>) -> bool {
    db.key_may_exist(id)
}
