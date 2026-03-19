locals {
  common_tags = {
    project     = var.project_name
    environment = var.environment
    managed_by  = "terraform"
  }
}

terraform {
  required_version = ">= 1.6.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.29"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

provider "kubernetes" {
  host                   = var.kubernetes_host != "" ? var.kubernetes_host : null
  token                  = var.kubernetes_token != "" ? var.kubernetes_token : null
  cluster_ca_certificate = var.kubernetes_ca_certificate != "" ? base64decode(var.kubernetes_ca_certificate) : null
}

resource "aws_s3_bucket" "service_files" {
  bucket = "${var.project_name}-${var.environment}-files"
  tags   = local.common_tags
}

resource "aws_s3_bucket_versioning" "service_files_versioning" {
  bucket = aws_s3_bucket.service_files.id
  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_public_access_block" "service_files_block_public" {
  bucket                  = aws_s3_bucket.service_files.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_secretsmanager_secret" "backend_secret" {
  name = "${var.project_name}/${var.environment}"
  tags = local.common_tags
}

resource "aws_secretsmanager_secret_version" "backend_secret_values" {
  secret_id = aws_secretsmanager_secret.backend_secret.id
  secret_string = jsonencode({
    DATABASE_URL          = var.database_url
    AWS_ACCESS_KEY_ID     = var.aws_access_key_id
    AWS_SECRET_ACCESS_KEY = var.aws_secret_access_key
  })
}

resource "kubernetes_namespace" "service_processes" {
  metadata {
    name = "service-processes"
    labels = {
      app = var.project_name
    }
  }
}

resource "kubernetes_config_map" "backend_config" {
  metadata {
    name      = "backend-config"
    namespace = kubernetes_namespace.service_processes.metadata[0].name
  }

  data = {
    RUST_LOG     = "info"
    S3_BUCKET    = aws_s3_bucket.service_files.bucket
    S3_REGION    = var.aws_region
    AWS_SECRET_NAME = aws_secretsmanager_secret.backend_secret.name
    NEXUS_URL    = var.nexus_url
    RABBITMQ_URL = var.rabbitmq_url
  }
}

resource "kubernetes_manifest" "rabbitmq" {
  manifest = {
    apiVersion = "apps/v1"
    kind       = "Deployment"
    metadata = {
      name      = "rabbitmq"
      namespace = kubernetes_namespace.service_processes.metadata[0].name
      labels = {
        app = "rabbitmq"
      }
    }
    spec = {
      replicas = 1
      selector = {
        matchLabels = {
          app = "rabbitmq"
        }
      }
      template = {
        metadata = {
          labels = {
            app = "rabbitmq"
          }
        }
        spec = {
          containers = [{
            name  = "rabbitmq"
            image = "rabbitmq:3.13-management"
            ports = [
              { containerPort = 5672 },
              { containerPort = 15672 }
            ]
          }]
        }
      }
    }
  }
}
