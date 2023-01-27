use std::collections::HashMap;

use parking_lot::RwLock;
use fluent::{bundle::FluentBundle, FluentResource};
use intl_memoizer::concurrent::IntlLangMemoizer;
use tracing::info;

pub struct FluentMessage{
    pub fluent_key: RwLock<String>,
    pub fluents: RwLock<HashMap<String, FluentBundle<FluentResource, IntlLangMemoizer>>>,
    pub fluent_def: RwLock<FluentBundle<FluentResource, IntlLangMemoizer>>,
}
impl FluentMessage{
    pub fn has_message(&self,key:&str) -> bool {
        for (key,flu) in self.fluents.read().iter() {
            if key==self.fluent_key.read().as_str(){
                return flu.has_message(key);
            }
       }
       return self.fluent_def.read().has_message(key);
    }
    pub fn get_message(&self, id: &str,args: Option<&fluent::FluentArgs>) -> String
    {
        for (key,flu) in self.fluents.read().iter() {
            if key==self.fluent_key.read().as_str(){
                match flu.get_message(id){
                    Some(msg)=>{
                        let pattern = msg.value();
                        let b=match pattern {
                            Some(value) => {
                                let mut errors = vec![];
                                let value = flu.format_pattern(value, args, &mut errors);
                                if !errors.is_empty(){
                                    info!("fluent_error[{}]:{:?}",id,errors)
                                }
                                value.to_string()
                            }
                            None =>id.to_string()
                        };
                        return b;
                    }
                    None =>{
                        return id.to_string();
                    }
                }
            }
       }
       let flu=self.fluent_def.read();
       match flu.get_message(id){
            Some(msg)=>{
                let pattern = msg.value();
                let b=match pattern {
                    Some(value) => {
                        let mut errors = vec![];
                        let value = flu.format_pattern(value, args, &mut errors);
                        if !errors.is_empty(){
                            info!("fluent_error[{}]:{:?}",id,errors)
                        }
                        value.to_string()
                    }
                    None =>id.to_string()
                };
                b
            }
            None =>{
                id.to_string()
            }
        }
    }
    pub fn set_lang(&self, lang: &str) {
        *self.fluent_key.write()=lang.to_owned();
    }
    pub fn set_message(&self, id: String,msg:String) {
        let ftl_string = format!("{} = {}",id,msg);
        if let Ok(resource) = FluentResource::try_new(ftl_string){
            for (key,flu) in self.fluents.write().iter_mut() {
                if key==self.fluent_key.read().as_str(){
                    if let Err(err)=flu.add_resource(resource){
                        tracing::error!("add default message fial: {:?}",err);
                    }
                    return;
                }
            }
            if let Err(err)=self.fluent_def.write().add_resource(resource){
                tracing::error!("add default message fial: {:?}",err);
            }
        }       
    }
}
