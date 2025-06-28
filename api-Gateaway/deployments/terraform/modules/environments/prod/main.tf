module "network" {
  source      = "../../modules/network"
  environment = "prod"
  vpc_cidr    = "10.1.0.0/16"
  azs         = ["us-east-1a", "us-east-1b", "us-east-1c"]
  public_subnet_cidrs  = ["10.1.1.0/24", "10.1.2.0/24", "10.1.3.0/24"]
  private_subnet_cidrs = ["10.1.4.0/24", "10.1.5.0/24", "10.1.6.0/24"]
}

module "redis" {
  source            = "../../modules/redis"
  environment       = "prod"
  subnet_ids        = module.network.private_subnet_ids
  security_group_id = module.network.security_group_id
  node_type         = "cache.m6g.large"
  redis_password    = var.redis_password
}

module "gateway" {
  source             = "../../modules/gateway"
  environment        = "prod"
  subnet_ids         = module.network.public_subnet_ids
  security_group_id  = module.network.security_group_id
  container_image    = var.container_image
  desired_count      = 6
  redis_host         = "${module.redis.redis_endpoint}:6379"
  target_group_arn   = module.nginx.target_group_arn
}

module "nginx" {
  source             = "../../modules/nginx"
  environment        = "prod"
  vpc_id             = module.network.vpc_id
  public_subnet_ids  = module.network.public_subnet_ids
  security_group_id  = module.network.security_group_id
  acm_cert_arn       = var.acm_cert_arn
}

module "monitoring" {
  source             = "../../modules/monitoring"
  environment        = "prod"
  vpc_id             = module.network.vpc_id
  security_group_id  = module.network.security_group_id
}