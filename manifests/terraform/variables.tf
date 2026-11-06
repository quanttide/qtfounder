variable "region" {
  description = "阿里云地域"
  type        = string
  default     = "cn-hangzhou"
}

variable "project" {
  description = "项目名（资源命名前缀）"
  type        = string
  default     = "qtfounder"
}

variable "environment" {
  description = "环境：dev / prod"
  type        = string
  default     = "prod"
}

variable "code_bucket" {
  description = "ZIP 代码包所在 OSS 桶（FC custom runtime）"
  type        = string
  default     = "qtcloud-asset"
}

variable "code_object" {
  description = "ZIP 代码包在桶内的对象路径"
  type        = string
  default     = "qtfounder/provider/qtfounder-provider-go-linux-amd64.zip"
}

variable "fc_memory" {
  description = "FC 函数内存（MB）"
  type        = number
  default     = 512
}

variable "fc_timeout" {
  description = "FC 函数超时（秒）"
  type        = number
  default     = 60
}
