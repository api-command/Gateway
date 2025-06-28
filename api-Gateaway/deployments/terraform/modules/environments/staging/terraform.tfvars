module "network" {
  source      = "../../modules/network"
  environment = "staging"
  vpc_cidr    = var.vpc_cidr
  azs         = var.azs
  public_subnet_cidrs  = var.public_subnet_cidrs
  private_subnet_cidrs = var.private_subnet_cidrs
}

module "redis" {
  source            = "../../modules/redis"
  environment       = "staging"
  subnet_ids        = module.network.private_subnet_ids
  security_group_id = module.network.security_group_id
  node_type         = "cache.t3.medium"
  redis_password    = var.redis_password
}

module "gateway" {
  source             = "../../modules/gateway"
  environment        = "staging"
  subnet_ids         = module.network.public_subnet_ids
  security_group_id  = module.network.security_group_id
  container_image    = var.container_image
  desired_count      = 3
  redis_host         = "${module.redis.redis_endpoint}:6379"
  target_group_arn   = module.nginx.target_group_arn
}

module "nginx" {
  source             = "../../modules/nginx"
  environment        = "staging"
  vpc_id             = module.network.vpc_id
  public_subnet_ids  = module.network.public_subnet_ids
  security_group_id  = module.network.security_group_id
  acm_cert_arn       = var.acm_cert_arn
}

module "monitoring" {
  source             = "../../modules/monitoring"
  environment        = "staging"
  vpc_id             = module.network.vpc_id
  security_group_id  = module.network.security_group_id
}