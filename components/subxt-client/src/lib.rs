use substrate_subxt::{
    events::Raw,
    Client, EventTypeRegistry, EventsDecoder, RawEvent,
    Runtime,ClientBuilder,
};
use crate::error::*;

pub mod error;
pub mod rpc;


pub struct SubstrateEvents<T: Runtime> {
    pub decoder: EventsDecoder<T>,
    client: Client<T>,
}

impl<T: Runtime> Clone for SubstrateEvents<T> {
    fn clone(&self) -> Self {
        SubstrateEvents::<T>::new(self.client.clone())
    }
}

impl<T: Runtime> SubstrateEvents<T> {
    pub fn new(client: Client<T>) -> Self {
        let event_registry = EventTypeRegistry::<T>::new();
        let decoder = EventsDecoder::<T>::new(client.metadata().clone(),event_registry);
        Self {
            decoder:decoder,
            client:client,
        }

    }

    pub fn decode_events(&self, input: &mut &[u8]) -> Result<Vec<RawEvent>> {
        let raw_events = self.decoder.decode_events(input)?;
        let mut events = vec![];
        for (_, raw) in raw_events {
            match raw {
                Raw::Event(event) => {
                    events.push(event);
                }
                Raw::Error(err) => {
                    log::error!("Error found in raw events: {:#?}", err);
                }
            }
        }
        Ok(events)
    }




}


pub struct SubstrateClient<T: Runtime>{
    pub subxt: Client<T>,
    pub event: SubstrateEvents<T>,
}

impl<T: Runtime> Clone for SubstrateClient<T> {
    fn clone(&self) -> Self {
        Self {
            subxt: self.subxt.clone(),
            event: self.event.clone(),
        }
    }
}

impl<T: Runtime> SubstrateClient<T> {
    pub async fn new(url: impl AsRef<str>) -> Result<SubstrateClient<T>> {
        let client = ClientBuilder::<T>::new()
        .set_url(url.as_ref())
        .skip_type_sizes_check()
        .build()
        .await?;
        let event = SubstrateEvents::<T>::new(client.clone());

        Ok(Self {
            subxt: client,
            event,
        })
    }

    // pub async fn subscribe_events<E: Event<T>>(&self) -> Result<EventSubscription<'a,T: Runtime>>{
    //     let sub = self.subxt.subscribe_events().await?;
    //     let decoder = self.subxt.events_decoder();
    //     let mut sub = EventSubscription::<T>::new(sub, decoder);
    //     sub.filter_event::<E>();
    //     Ok(sub);
    // }


}