# 函数计算（FC 3.0）：custom.debian12 runtime + ZIP 代码包。
# 参考 qtcloud-asset-provider 模式（无 Docker/镜像仓库依赖）：
#   - Go 静态编译为 bootstrap，与数据源（fiction/memory）一起打 ZIP 上传 OSS
#   - custom runtime 经 custom_runtime_config 指定启动命令与监听端口（9000）与健康检查（/healthz）
#   - HTTP 触发器公开访问，鉴权双层：触发器 JWT（JWKS 在本目录 jwks.json）+ 应用层 JWTAuth 中间件
resource "alicloud_fcv3_function" "this" {
  function_name        = "${var.project}-${var.environment}"
  description          = "qtfounder 创作数据 API"
  runtime              = "custom.debian12"
  handler              = "not-used"
  memory_size          = var.fc_memory
  cpu                  = 0.5
  disk_size            = 512
  timeout              = var.fc_timeout
  instance_concurrency = 10
  internet_access      = true

  code {
    oss_bucket_name = var.code_bucket
    oss_object_name = var.code_object
  }

  custom_runtime_config {
    command = ["./bootstrap"]
    port    = 9000

    # 健康检查探 /healthz（应用层免鉴权路径）；不显式声明会漂移回默认的 /health
    health_check_config {
      http_get_url          = "/healthz"
      initial_delay_seconds = 1
      period_seconds        = 10
      timeout_seconds       = 3
      success_threshold     = 1
      failure_threshold     = 3
    }
  }

  # 访问控制：应用层 JWTAuth 中间件（internal/creative/auth.go）复验 RS256 令牌，
  # 客户端以 Authorization: Bearer <JWT> 访问（cmd/mint 用本机私钥签发）。
  # 公钥为公开信息，直接进配置；私钥不落仓库、不进 tfstate。
  environment_variables = {
    QTFOUNDER_JWT_PUBLIC_KEY = file("${path.module}/jwt_public_key.pem")
    QTFOUNDER_ADDR           = ":9000"
    QTFOUNDER_FICTION_PATH   = "./data/fiction"
    QTFOUNDER_MEMORY_PATH    = "./data/memory"
  }

  tags = {
    project     = var.project
    environment = var.environment
  }
}

# HTTP 触发器：边缘 JWT 鉴权（authType=jwt）。触发器用 jwks.json 的公钥验签，
# tokenLookup 取 Authorization 头并去掉 "Bearer " 前缀；/healthz 白名单免验（健康检查与探活）。
# 配置格式与 serverless-devs/fc3 的 convertIHttpTriggerConfig 输出对齐。
resource "alicloud_fcv3_trigger" "http" {
  function_name = alicloud_fcv3_function.this.function_name
  trigger_name  = "http"
  trigger_type  = "http"
  qualifier     = "LATEST"
  trigger_config = jsonencode({
    authType = "jwt"
    methods  = ["GET", "OPTIONS"]
    authConfig = {
      jwks        = jsondecode(file("${path.module}/jwks.json"))
      tokenLookup = "header:Authorization:Bearer "
      claimPassBy = "query::"
      whitelist   = ["/healthz"]
    }
  })
}
