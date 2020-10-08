use std::collections::{HashSet, HashMap};
use uuid::Uuid;
use std::str::FromStr;

#[derive(Clone)]
pub struct Decision {
    choice: String,
    rationale: String,
    decision_makers: HashSet<String>,
}

#[derive(Clone)]
pub struct Question {
    identifier: Uuid,
    content: String,
    tags: HashSet<String>,
    context: HashSet<String>,
    options: HashSet<String>,
    decision: Option<Decision>
}

pub enum SetDecisionError {
    AlreadyExists
}

impl Question {
    fn new(content: String, tags: HashSet<String>, context: HashSet<String>, options: HashSet<String>) -> Question {
        Question {
            identifier: Uuid::new_v4(),
            content,
            tags,
            context,
            options,
            decision: None
        }
    }

    fn add_context(&mut self, context_item: String){
        self.context.insert(context_item);
    }

    fn get_context(&self) -> HashSet<String>{
        self.context.clone()
    }

    fn add_option(&mut self, option: String){
        self.options.insert(option);
    }

    fn get_options(&self) -> HashSet<String>{
        self.options.clone()
    }

    fn set_decision(&mut self, decision: Decision) -> Result<(), SetDecisionError>{
        match self.decision {
            None => {
                self.decision = Some(decision);
                Result::Ok(())
            }
            Some(_) => return Result::Err(SetDecisionError::AlreadyExists)
        }
    }

    fn get_decision(&self) -> Option<Decision> {
        self.decision.clone()
    }
}

pub struct Registry {
    tags: HashSet<String>,
    questions: HashMap<Uuid, Question>,
}

#[derive(Debug)]
pub enum AddTagErrors {
    AlreadyExists
}
#[derive(Debug)]
pub enum AddQuestionError {
    AlreadyExists,
    UsesNonExistentTags(Vec<String>)
}
#[derive(Debug)]
pub enum GetQuestionError {
    InvalidUUID,
    DoesNotExist
}

impl Registry {
    pub fn new() -> Registry {
        Registry {
            tags: Default::default(),
            questions: Default::default()
        }
    }

    pub fn add_tag(&mut self, tag: &String) -> Result<bool, AddTagErrors> {
        return if self.tags.contains(tag) {
            Result::Err(AddTagErrors::AlreadyExists)
        } else {
            self.tags.insert(tag.clone());
            Result::Ok(true)
        }
    }

    pub fn get_tags(&self) -> HashSet<String> {
        return self.tags.clone()
    }

    pub fn add_question(&mut self, question: Question) -> Result<String, AddQuestionError> {
        let tag_diff: HashSet<_>= question.tags.difference(&self.tags).collect();
        if !tag_diff.is_empty() {
            let mut response : Vec<String> = Vec::new();
            for tag in tag_diff.into_iter().enumerate() {
                response.push(String::from(tag.1));
            }

            return Result::Err(AddQuestionError::UsesNonExistentTags(response));
        }
        return if self.questions.contains_key(&question.identifier){
            Result::Err(AddQuestionError::AlreadyExists)
        } else {
            let identifier = question.identifier.to_string();
            self.questions.insert(question.identifier, question);
            Result::Ok(identifier)
        }
    }

    fn get_question(&self, identifier: String) -> Result<Question, GetQuestionError> {
        match Uuid::from_str(&identifier) {
            Ok(uuid) => {
                match self.questions.get(&uuid) {
                    Some(question) => Result::Ok((*question).clone()),
                    _ => Result::Err(GetQuestionError::DoesNotExist)
                }
            }
            _ => Result::Err(GetQuestionError::InvalidUUID)
        }
    }

    fn add_question_context(&self, identifier: String, new_contexts: HashSet<String>){
        match self.get_question(identifier) {
            Ok(mut question) => {
                new_contexts.iter().for_each(|context| question.add_context(context.to_string()))
            },
            Err(_) => ()
        }
    }

    fn add_question_option(&self, identifier: String, new_options: HashSet<String>){
        match self.get_question(identifier) {
            Ok(mut question) => {
                new_options.iter().for_each(|context| question.add_option(context.to_string()))
            },
            Err(_) => ()
        }
    }

    fn set_question_decision(&self, identifier: String, decision: Decision){
        match self.get_question {
            Ok(mut question) => {
                question.set_decision(decision)
            },
            Err(_) => ()
        }
    }

}

#[cfg(test)]
mod tests {
    use crate::*;

    const TAG_A : &str = "Luke'sFunProjectA";
    const TAG_B : &str = "Luke'sOtherFunProject";
    const TAG_C : &str = "AriesThing";
    const TAG_D : &str = "AdasEndeavor";

    fn add_some_default_tags(registry: &mut Registry) -> () {
        [TAG_A, TAG_B, TAG_C, TAG_D].iter().for_each(|x| {
            registry.add_tag(&String::from(*x));
        });
    }

    #[test]
    fn test_add_tag() {
        let mut registry = Registry::new();
        let tag_value = "Something".to_string();
        registry.add_tag(&tag_value);
        assert!(registry.get_tags().contains(&tag_value));
    }

    #[test]
    fn test_add_existing_tag_fails() {
        let mut registry = Registry::new();
        let tag_value = "Something".to_string();
        registry.add_tag(&tag_value);

        assert!(registry.add_tag(&tag_value).is_err(), "This should have failed due to tag already existing")
    }

    #[test]
    fn test_add_and_question() -> Result<(), AddQuestionError> {
        let mut registry = Registry::new();
        add_some_default_tags(&mut registry);
        let mut question_tags : HashSet<String> = HashSet::new();
        question_tags.insert(TAG_A.parse().unwrap());
        let question = Question::new("How many tests will luke end up writing?".to_string(),
                                     question_tags,
                                     HashSet::new(),
                                     HashSet::new());
        let identifier = registry.add_question(question)?;
        Result::Ok(())
    }

    #[test]
    fn test_question_with_nonexistent_tag_wont_work(){
        let fake_project_name = String::from("ThisIsn'tOneOfMyProjects!");
        let mut registry = Registry::new();
        add_some_default_tags(&mut registry);
        let mut question_tags : HashSet<String> = HashSet::new();
        question_tags.insert(fake_project_name.clone());
        let question = Question::new("How many tests will luke end up writing?".to_string(),
                                     question_tags,
                                     HashSet::new(),
                                     HashSet::new());
        match registry.add_question(question) {
            Ok(_) => panic!("This should have never worked!"),
            Err(AddQuestionError::UsesNonExistentTags(tags)) => assert!(tags.contains(&fake_project_name)),
            _ => panic!("Got an add question error we did not expect")
        }
    }

    #[test]
    fn test_adding_context_to_question(){
        let mut registry = Registry::new();
        add_some_default_tags(&mut registry);
        let mut question_tags : HashSet<String> = HashSet::new();
        question_tags.insert(TAG_A.parse().unwrap());
        let question = Question::new("How many tests will luke end up writing?".to_string(),
                                     question_tags,
                                     HashSet::new(),
                                    HashSet::new());
        let identifier = registry.add_question(question);
    }

}
