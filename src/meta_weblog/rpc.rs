use std::fs::File;

use chrono::Timelike;
use chrono::Datelike;
use crate::meta_weblog::weblog::WpCategory;
use crate::BlogInfo;
use crate::CategoryInfo;
use iso8601::DateTime;
use xmlrpc::{Error, Request, Value};

use super::weblog::Post;

const DELETE_POST: &str = "blogger.deletePost";
const EDIT_POST: &str = "metaWeblog.editPost";
const GET_CATEGORIES: &str = "metaWeblog.getCategories";
const GET_POST: &str = "metaWeblog.getPost";
const GET_RECENT_POSTS: &str = "metaWeblog.getRecentPosts";
const GET_USERS_BLOGS: &str = "blogger.getUsersBlogs";
const NEW_POST: &str = "metaWeblog.newPost";
const NEW_CATEGORY: &str = "wp.newCategory";
const SERVER_URL: &str = "https://rpc.cnblogs.com/metaweblog";

pub struct MetaWeblog {
    app_key: String,
    username: String,
    password: String,
    blogid: String,
    url: String,
}

impl MetaWeblog {
    // new
    pub fn new(username: String, password: String, blogid: String, app_key: String) -> Self {
        MetaWeblog {
            url: format!("{}/{}", SERVER_URL, app_key.as_str()),
            password,
            blogid,
            app_key,
            username,
        }
    }

    pub fn new_post(&self, mut post: Post, publish: bool) -> Result<String, Error> {
        if post.dateCreated == DateTime::default() {
            post.dateCreated = Self::get_now_time();
        }
        // 1. geerate arguments
        let mut arguments = Vec::<Value>::new();
        arguments.push(Value::String(self.blogid.to_string()));
        arguments.push(Value::String(self.username.to_string()));
        arguments.push(Value::String(self.password.to_string()));
        arguments.push(post.into());
        arguments.push(Value::Bool(publish));

        // 2. call rpc
        let result = self.rpc_request(NEW_POST, arguments)?;

        // 3. parse result
        if let Value::String(postid) = result {
            return Ok(postid);
        }
        Ok("-2".to_string())
    }

    pub fn new_category(&self, category: WpCategory) -> Result<i32, Error> {
        // 1. geerate arguments
        let mut arguments = Vec::<Value>::new();
        arguments.push(Value::String(self.blogid.to_string()));
        arguments.push(Value::String(self.username.to_string()));
        arguments.push(Value::String(self.password.to_string()));
        arguments.push(category.into());

        // 2. call rpc
        let result = self.rpc_request(NEW_CATEGORY, arguments)?;

        // 3. parse result
        if let Value::Int(categoryid) = result {
            return Ok(categoryid);
        }
        Ok(-1)
    }

    pub fn get_post(&self, postid: &str) -> Result<Post, Error> {
        // 1. geerate arguments
        let mut arguments = Vec::<Value>::new();
        arguments.push(Value::String(postid.to_string()));
        arguments.push(Value::String(self.username.to_string()));
        arguments.push(Value::String(self.password.to_string()));

        // 2. call rpc
        let result = self.rpc_request(GET_POST, arguments)?;

        // 3. parse result
        let post = Post::from(result);
        Ok(post)
    }

    pub fn get_recent_posts(&self, num: u32) -> Result<Vec<Post>, Error> {
        // 1. geerate arguments
        let mut arguments = Vec::<Value>::new();
        arguments.push(Value::String(self.blogid.to_string()));
        arguments.push(Value::String(self.username.to_string()));
        arguments.push(Value::String(self.password.to_string()));
        arguments.push(Value::Int(num as i32));

        // 2. call rpc
        let result = self.rpc_request(GET_RECENT_POSTS, arguments)?;

        // 3. parse result
        let mut posts = Vec::<Post>::new();
        if let Value::Array(results) = result {
            for v in results.into_iter() {
                posts.push(v.into());
            }
        }
        Ok(posts)
    }

    pub fn get_categories(&self) -> Result<Vec<CategoryInfo>, Error> {
        // 1. generate arguments
        let mut args = Vec::<Value>::new();
        args.push(Value::String(self.blogid.to_string()));
        args.push(Value::String(self.username.to_string()));
        args.push(Value::String(self.password.to_string()));

        // 2. call url
        let result = self.rpc_request(GET_CATEGORIES, args)?;

        // 3. parse result
        let mut categories = Vec::<CategoryInfo>::new();
        if let Value::Array(results) = result {
            for v in results.into_iter() {
                let category = CategoryInfo::from(v);
                categories.push(category);
            }
        }
        Ok(categories)
    }

    pub fn get_users_blogs(&self) -> Result<Vec<BlogInfo>, Error> {
        // 1. generate arguments
        let mut args = Vec::<Value>::new();
        args.push(Value::String(self.app_key.clone()));
        args.push(Value::String(self.username.clone()));
        args.push(Value::String(self.password.clone()));

        // 2. call rpc
        let result = self.rpc_request(GET_USERS_BLOGS, args)?;

        // 3. parse result
        let mut blog_infos = Vec::<BlogInfo>::new();
        if let Value::Array(results) = result {
            for v in results {
                let blog_info = BlogInfo::from(v);
                blog_infos.push(blog_info);
            }
        }
        Ok(blog_infos)
    }

    pub fn edit_post(&self, postid: &str, mut post: Post, publish: bool) -> Result<Value, Error> {
        if post.dateCreated == DateTime::default() {
            post.dateCreated = Self::get_now_time();
        }
        // 1. generate parameters
        let mut arguments = Vec::<Value>::new();
        arguments.push(Value::String(postid.to_string()));
        arguments.push(Value::String(self.username.to_string()));
        arguments.push(Value::String(self.password.to_string()));
        arguments.push(post.into());
        arguments.push(Value::Bool(publish));

        // 2. call rpc
        let result = self.rpc_request(EDIT_POST, arguments)?;

        // 3. parse result
        Ok(result)
    }

    /// Delete post by postid
    pub fn delete_post(&self, postid: &str, publish: bool) -> Result<bool, Error> {
        // 1. generate arguments
        let mut arguments = Vec::<Value>::new();
        arguments.push(Value::String(self.app_key.clone()));
        arguments.push(Value::String(postid.to_string()));
        arguments.push(Value::String(self.username.to_string()));
        arguments.push(Value::String(self.password.clone()));
        arguments.push(Value::Bool(publish));

        // 2. call rpc
        let result = self.rpc_request(DELETE_POST, arguments)?;

        // 3. parse result
        if let Value::Bool(v) = result {
            return Ok(v);
        }
        Ok(false)
    }

    fn rpc_request(&self, method: &str, args: Vec<Value>) -> Result<Value, Error> {
        // When `request` call `arg()`, owenership entry function. So we need rereceive
        let mut request = Request::new(method);

        for arg in args.into_iter() {
            request = request.arg(arg);
        }
        let mut f = File::create("1.xml").unwrap();
        
        request.write_as_xml(&mut f).unwrap();
        request.call_url(self.url.as_str())
    }

    fn get_now_time() -> DateTime {
        let now = chrono::Local::now();
        let s = format!("{}-{:02}-{:02}T{:02}:{:02}:{:02}", now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second());
        iso8601::datetime(s.as_str()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, Timelike};

    use super::{WpCategory, MetaWeblog};

    #[test]
    fn get_users_blogs() {
        let weblog = MetaWeblog::new("上海的海是海未的海".to_string(), "63F4E40156E9BCE22EC53B951D1ED9D6D2855218E78DB9AE338B7FF63123BC0E".to_string(), "123".to_string(), "lunar-umi".to_string());
        let a = weblog.get_users_blogs().unwrap();
        dbg!(a);
    }
    #[test]
    fn new_category() {
        let weblog = MetaWeblog::new("上海的海是海未的海".to_string(), "63F4E40156E9BCE22EC53B951D1ED9D6D2855218E78DB9AE338B7FF63123BC0E".to_string(), "123".to_string(), "lunar-umi".to_string());
        let mut category = WpCategory::default();
        category.name = "Cates".to_string();
        let a = weblog.new_category(category).unwrap();
        dbg!(a);
    }

    #[test]
    fn get_recent_posts() {
        let weblog = MetaWeblog::new("上海的海是海未的海".to_string(), "63F4E40156E9BCE22EC53B951D1ED9D6D2855218E78DB9AE338B7FF63123BC0E".to_string(), "123".to_string(), "lunar-umi".to_string());
        let posts = weblog.get_recent_posts(100).unwrap();
        println!("{:?}", posts);
    }

    #[test]
    fn delete_all_posts() {
        let weblog = MetaWeblog::new("上海的海是海未的海".to_string(), "63F4E40156E9BCE22EC53B951D1ED9D6D2855218E78DB9AE338B7FF63123BC0E".to_string(), "123".to_string(), "lunar-umi".to_string());
        let posts = weblog.get_recent_posts(999).unwrap();
        for post in posts {
            println!("{:?}", post);
            weblog.delete_post(post.postid.as_str(), true).unwrap();
        }
    }

    #[test]
    fn delete_post() {
        let weblog = MetaWeblog::new("上海的海是海未的海".to_string(), "63F4E40156E9BCE22EC53B951D1ED9D6D2855218E78DB9AE338B7FF63123BC0E".to_string(), "123".to_string(), "lunar-umi".to_string());
        let posts = weblog.delete_post("16252136",true).unwrap();
        println!("{:?}", posts);
    }

    #[test]
    fn time_test() {
        let now = chrono::Local::now();
        let mut post = crate::Post::default();
        let s = format!("{}-{:02}-{:02}T{:02}:{:02}:{:02}", now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second());
        post.dateCreated = iso8601::datetime(s.as_str()).unwrap();
        println!("{}", s);
    }
}
