variable "project_name" {
  type    = string
  default = "service-processes"
}

variable "environment" {
  type    = string
  default = "dev"
}

variable "aws_region" {
  type    = string
  default = "eu-central-1"
}

variable "database_url" {
  type      = string
  sensitive = true
}

variable "aws_access_key_id" {
  type      = string
  sensitive = true
}

variable "aws_secret_access_key" {
  type      = string
  sensitive = true
}

variable "rabbitmq_url" {
  type    = string
  default = "amqp://rabbitmq.service-processes.svc.cluster.local:5672"
}

variable "nexus_url" {
  type    = string
  default = "http://nexus.service-processes.svc.cluster.local:8081"
}
