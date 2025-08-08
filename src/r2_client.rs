use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, Method,
};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

pub struct R2Client {
    client: Client,
    access_key_id: String,
    secret_access_key: String,
    account_id: String,
    bucket_name: String,
    endpoint: String,
}

impl R2Client {
    pub async fn new(
        access_key_id: String,
        secret_access_key: String,
        account_id: String,
        bucket_name: String,
    ) -> Result<Self> {
        let endpoint = format!("https://{}.r2.cloudflarestorage.com", account_id);

        Ok(Self {
            client: Client::new(),
            access_key_id,
            secret_access_key,
            account_id,
            bucket_name,
            endpoint,
        })
    }

    fn sign_request(
        &self,
        method: &Method,
        path: &str,
        headers: &mut HeaderMap,
        payload: &[u8],
        datetime: &DateTime<Utc>,
    ) -> Result<()> {
        let date_str = datetime.format("%Y%m%dT%H%M%SZ").to_string();
        let date_short = datetime.format("%Y%m%d").to_string();

        let payload_hash = hex::encode(Sha256::digest(payload));

        headers.insert("x-amz-date", HeaderValue::from_str(&date_str)?);
        headers.insert(
            "x-amz-content-sha256",
            HeaderValue::from_str(&payload_hash)?,
        );

        let host = format!("{}.r2.cloudflarestorage.com", self.account_id);
        headers.insert("host", HeaderValue::from_str(&host)?);

        // Extract query string from path if present
        let (path_only, query_string) = if let Some(pos) = path.find('?') {
            (&path[..pos], &path[pos + 1..])
        } else {
            (path, "")
        };

        let canonical_headers = format!(
            "host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}",
            host, payload_hash, date_str
        );

        let signed_headers = "host;x-amz-content-sha256;x-amz-date";

        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n\n{}\n{}",
            method.as_str(),
            path_only,  // Path is already properly encoded by the caller
            query_string,
            canonical_headers,
            signed_headers,
            payload_hash
        );

        let canonical_request_hash = hex::encode(Sha256::digest(canonical_request.as_bytes()));

        let credential_scope = format!("{}/auto/s3/aws4_request", date_short);

        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{}\n{}\n{}",
            date_str, credential_scope, canonical_request_hash
        );

        let mut key = format!("AWS4{}", self.secret_access_key).into_bytes();

        for item in [date_short.as_bytes(), b"auto", b"s3", b"aws4_request"] {
            let mut mac = HmacSha256::new_from_slice(&key)?;
            mac.update(item);
            key = mac.finalize().into_bytes().to_vec();
        }

        let mut mac = HmacSha256::new_from_slice(&key)?;
        mac.update(string_to_sign.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        let authorization = format!(
            "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
            self.access_key_id, credential_scope, signed_headers, signature
        );

        headers.insert("authorization", HeaderValue::from_str(&authorization)?);

        Ok(())
    }


    pub async fn download_object(&self, key: &str) -> Result<Bytes> {
        // Encode the key segments for both URL and canonical path
        let encoded_key = key.split('/').map(|s| urlencoding::encode(s)).collect::<Vec<_>>().join("/");
        // Build the path with encoded key for signing
        let path = format!("/{}/{}", self.bucket_name, encoded_key);
        // Build the URL
        let url = format!("{}{}", self.endpoint, path);

        let mut headers = HeaderMap::new();
        let datetime = Utc::now();

        self.sign_request(&Method::GET, &path, &mut headers, b"", &datetime)?;

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .context("Failed to download object from R2")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "R2 download failed with status {}: {}",
                status,
                error_text
            ));
        }

        let data = response
            .bytes()
            .await
            .context("Failed to read response body")?;

        Ok(data)
    }

    pub async fn upload_object(&self, key: &str, data: Bytes) -> Result<()> {
        // Encode the key segments for both URL and canonical path
        let encoded_key = key.split('/').map(|s| urlencoding::encode(s)).collect::<Vec<_>>().join("/");
        // Build the path with encoded key for signing
        let path = format!("/{}/{}", self.bucket_name, encoded_key);
        // Build the URL
        let url = format!("{}{}", self.endpoint, path);

        let mut headers = HeaderMap::new();
        let datetime = Utc::now();

        self.sign_request(&Method::PUT, &path, &mut headers, &data, &datetime)?;

        let response = self
            .client
            .put(&url)
            .headers(headers)
            .body(data)
            .send()
            .await
            .context("Failed to upload object to R2")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "R2 upload failed with status {}: {}",
                status,
                error_text
            ));
        }

        Ok(())
    }

    pub async fn list_objects(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        let query_params = if let Some(p) = prefix {
            format!("list-type=2&prefix={}", urlencoding::encode(p))
        } else {
            "list-type=2".to_string()
        };

        let path = format!("/{}?{}", self.bucket_name, query_params);
        let url = format!("{}{}", self.endpoint, path);

        let mut headers = HeaderMap::new();
        let datetime = Utc::now();

        self.sign_request(&Method::GET, &path, &mut headers, b"", &datetime)?;

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .context("Failed to list objects in R2")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "R2 list failed with status {}: {}",
                status,
                error_text
            ));
        }

        let xml_text = response.text().await?;

        let mut reader = quick_xml::Reader::from_str(&xml_text);
        let mut objects = Vec::new();
        let mut in_key = false;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(ref e)) if e.name().as_ref() == b"Key" => {
                    in_key = true;
                }
                Ok(quick_xml::events::Event::Text(ref e)) if in_key => {
                    objects.push(e.unescape()?.to_string());
                }
                Ok(quick_xml::events::Event::End(ref e)) if e.name().as_ref() == b"Key" => {
                    in_key = false;
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Err(e) => return Err(anyhow!("XML parsing error: {}", e)),
                _ => {}
            }
            buf.clear();
        }

        Ok(objects)
    }

    pub async fn delete_object(&self, key: &str) -> Result<()> {
        // Encode the key segments for both URL and canonical path
        let encoded_key = key.split('/').map(|s| urlencoding::encode(s)).collect::<Vec<_>>().join("/");
        // Build the path with encoded key for signing
        let path = format!("/{}/{}", self.bucket_name, encoded_key);
        // Build the URL
        let url = format!("{}{}", self.endpoint, path);

        let mut headers = HeaderMap::new();
        let datetime = Utc::now();

        self.sign_request(&Method::DELETE, &path, &mut headers, b"", &datetime)?;

        let response = self
            .client
            .delete(&url)
            .headers(headers)
            .send()
            .await
            .context("Failed to delete object from R2")?;

        if !response.status().is_success() && response.status().as_u16() != 404 {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "R2 delete failed with status {}: {}",
                status,
                error_text
            ));
        }

        Ok(())
    }
}

#[allow(dead_code)]
mod urlencoding {
    pub fn encode(s: &str) -> String {
        s.bytes()
            .map(|byte| {
                if byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_' || byte == b'.' || byte == b'~' {
                    char::from(byte).to_string()
                } else {
                    format!("%{:02X}", byte)
                }
            })
            .collect()
    }
}
