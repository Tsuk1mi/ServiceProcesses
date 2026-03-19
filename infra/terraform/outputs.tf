output "s3_bucket_name" {
  value       = aws_s3_bucket.service_files.bucket
  description = "S3 bucket for attachments and artifacts"
}

output "aws_secret_name" {
  value       = aws_secretsmanager_secret.backend_secret.name
  description = "AWS Secrets Manager secret name used by backend"
}

output "k8s_namespace" {
  value       = kubernetes_namespace.service_processes.metadata[0].name
  description = "Kubernetes namespace for the project"
}

output "s3_bucket_arn" {
  value       = aws_s3_bucket.service_files.arn
  description = "S3 bucket ARN"
}

output "rabbitmq_deployment_name" {
  value       = "rabbitmq"
  description = "RabbitMQ deployment name managed by Terraform"
}
