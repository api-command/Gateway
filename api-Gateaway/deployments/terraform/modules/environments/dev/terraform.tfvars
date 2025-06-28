environment    = "dev"
vpc_cidr       = "10.0.0.0/16"
azs            = ["us-east-1a", "us-east-1b"]
public_subnet_cidrs  = ["10.0.1.0/24", "10.0.2.0/24"]
private_subnet_cidrs = ["10.0.3.0/24", "10.0.4.0/24"]

# Example values - replace with actual ARN/password
acm_cert_arn   = "arn:aws:acm:us-east-1:123456789012:certificate/xxxxxx"
redis_password = "dev-redis-password-123"