import boto3
from botocore.config import Config

# R2 credentials
access_key_id = '9dd87b553748932096940460cf045150'
secret_access_key = 'e78e0ff78b474d1d4f9d01dff1f5fb569abcea715eca1eeed42f55415c990e83'
account_id = '72a61d050034cb73f26694a75073f83a'

# Create S3 client with R2 endpoint
s3 = boto3.client(
    's3',
    endpoint_url=f'https://{account_id}.r2.cloudflarestorage.com',
    aws_access_key_id=access_key_id,
    aws_secret_access_key=secret_access_key,
    region_name='auto',
    config=Config(signature_version='s3v4')
)

try:
    # List buckets
    response = s3.list_buckets()
    print("Buckets:")
    for bucket in response.get('Buckets', []):
        print(f"  - {bucket['Name']}")
except Exception as e:
    print(f"Error: {e}")
