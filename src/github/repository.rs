use std::path::{Path, PathBuf};

use futures::{stream, Stream, StreamExt, TryStreamExt};
use gloo::net::http::Request;
use octocrab::models;
use url::Url;

use crate::{
    error::{
        repository::{Unreachable, UrlParseError},
        Result,
    },
    github::models::{ContentsType, SubtreeModel, TreesModel},
};

use super::{blob::GitHubBlob, statistics::Statistics};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubRepository {
    pub owner: String,
    pub repo: String,
}

impl GitHubRepository {
    pub const ORIGIN: &'static str = "https://github.com";
    pub const API_ORIGIN: &'static str = "https://api.github.com";
    pub const RAW_ORIGIN: &'static str = "https://raw.githubusercontent.com";

    pub fn new(owner: &str, repo: &str) -> Self {
        Self { owner: owner.to_string(), repo: repo.to_string() }
    }

    pub fn host(&self) -> String {
        "github".to_string()
    }

    pub fn from_url(url: &Url) -> Result<Self> {
        if url.origin().unicode_serialization() != Self::ORIGIN {
            Err(anyhow::anyhow!(UrlParseError::InvalidHost))?
        }
        let mut path_segments = url.path_segments().ok_or_else(|| anyhow::anyhow!(UrlParseError::Unspecified))?;
        let owner = path_segments.next().ok_or_else(|| anyhow::anyhow!(UrlParseError::UnspecifiedOwner))?;
        let repo = path_segments.next().ok_or_else(|| anyhow::anyhow!(UrlParseError::UnspecifiedRepository))?;
        // TODO rest path
        Ok(Self::new(owner, repo))
    }

    pub fn to_url(&self) -> Result<Url> {
        let mut url = Url::parse(Self::ORIGIN).map_err(anyhow::Error::from)?;
        url.set_path(&[&self.owner[..], &self.repo[..]].join("/"));
        Ok(url)
    }

    pub fn api_endpoint(&self, path: &str) -> Result<Url> {
        let mut url = Url::parse(Self::API_ORIGIN).map_err(anyhow::Error::from)?;
        url.set_path(path);
        Ok(url)
    }

    pub fn raw_endpoint(&self, path: &str) -> Result<Url> {
        let mut url = Url::parse(Self::RAW_ORIGIN).map_err(anyhow::Error::from)?;
        url.set_path(path);
        Ok(url)
    }

    pub async fn trees(&self, sha: &str, recursive: bool) -> Result<TreesModel> {
        let Self { owner, repo } = &self;
        let path = format!("/repos/{owner}/{repo}/git/trees/{sha}");
        let request = Request::get(self.api_endpoint(&path)?.as_str()).query([("recursive", recursive.to_string())]);
        Ok(request.send().await.map_err(anyhow::Error::from)?.json().await.map_err(anyhow::Error::from)?)
    }

    pub async fn repository(&self) -> Result<models::Repository> {
        let Self { owner, repo } = &self;
        let path = format!("/repos/{owner}/{repo}");
        let request = Request::get(self.api_endpoint(&path)?.as_str());
        Ok(request.send().await.map_err(anyhow::Error::from)?.json().await.map_err(anyhow::Error::from)?)
    }

    pub async fn raw<A: AsRef<Path>>(&self, sha: &str, path: A) -> Result<String> {
        let Self { owner, repo } = &self;
        let path = path.as_ref().to_str().ok_or_else(|| anyhow::anyhow!(Unreachable::UnimplementedString))?;
        let path = format!("/{owner}/{repo}/{sha}/{path}");
        let request = Request::get(self.raw_endpoint(&path)?.as_str());
        Ok(request.send().await.map_err(anyhow::Error::from)?.text().await.map_err(anyhow::Error::from)?)
    }

    pub async fn walk<'a>(&'a self, sha: &'a str) -> impl Stream<Item = Result<GitHubBlob>> + 'a {
        // TODO zip or tar.gz
        let TreesModel { tree, .. } = self.trees(&sha, true).await.unwrap();
        let paths = tree.into_iter().filter_map(|SubtreeModel { path, contents_type, .. }| match contents_type {
            ContentsType::Tree => None,
            ContentsType::Blob => Some(PathBuf::from(path)),
            ContentsType::Commit => None,
        });

        stream::iter(paths.clone())
            .map(move |path| self.raw(&sha, path))
            .buffered(32) // num_cpus::get() returns 1
            .zip(stream::iter(paths))
            .map(|(raw, path)| Ok(GitHubBlob::new(path, raw?)))
            .map_ok(|blob| blob)
    }

    pub async fn get_statistics(&self, config: &tokei::Config) -> Result<Statistics> {
        Statistics::get(self.clone(), config).await // TODO lifetime
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_repo_url() {
        let url = Url::parse("https://github.com/hayas1/tokei-toukei").unwrap();
        let repo = GitHubRepository::from_url(&url).unwrap();
        assert_eq!(repo, GitHubRepository::new("hayas1", "tokei-toukei"));
        assert_eq!(repo.to_url().unwrap().as_str(), "https://github.com/hayas1/tokei-toukei");

        let url = Url::parse("https://github.com/hayas1/tokei-toukei/").unwrap();
        let repo = GitHubRepository::from_url(&url).unwrap();
        assert_eq!(repo, GitHubRepository::new("hayas1", "tokei-toukei"));
        assert_eq!(repo.to_url().unwrap().as_str(), "https://github.com/hayas1/tokei-toukei");
    }
}
