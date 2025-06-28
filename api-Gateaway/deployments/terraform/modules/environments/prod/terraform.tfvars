# Networking
vpc_cidr       = "10.1.0.0/16"
azs            = ["us-east-1a", "us-east-1b", "us-east-1c"]
public_subnet_cidrs  = ["10.1.1.0/24", "10.1.2.0/24", "10.1.3.0/24"]
private_subnet_cidrs = ["10.1.4.0/24", "10.1.5.0/24", "10.1.6.0/24"]

# Security
acm_cert_arn   = "arn:aws:acm:us-east-1:123456789012:certificate/prod-cert-xyz"
redis_password = "prod-redis-password-!@#secure123"

# Deployment
container_image = "123456789012.dkr.ecr.us-east-1.amazonaws.com/api-gateway:v1.5.0"

# Monitoring
alert_email = "prod-alerts@company.com"