module "network" {
  source      = "../../modules/network"
  environment = "dev"
  vpc_cidr    = "10.0.0.0/16"
  azs         = ["us-east-1a", "us-east-1b"]
}

module "redis" {
  source            = "../../modules/redis"
  environment       = "dev"
  subnet_ids        = module.network.private_subnet_ids
  security_group_id = module.network.security_group_id
  node_type         = "cache.t3.micro"
}

module "gateway" {
  source             = "../../modules/gateway"
  environment        = "dev"
  subnet_ids         = module.network.public_subnet_ids
  security_group_id  = module.network.security_group_id
  container_image    = "your-registry/api-gateway:latest"
  desired_count      = 2
  redis_host         = module.redis.redis_endpoint
  target_group_arn   = module.nginx.target_group_arn
}

module "nginx" {
  source             = "../../modules/nginx"
  environment        = "dev"
  vpc_id             = module.network.vpc_id
  public_subnet_ids  = module.network.public_subnet_ids
  security_group_id  = module.network.security_group_id
  acm_cert_arn       = var.acm_cert_arn
}