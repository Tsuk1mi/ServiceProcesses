using System.Net.Http.Json;
using Frontend.Windows.Domain.Dto;

namespace Frontend.Windows.Infrastructure.Api;

/// <summary>
/// Централизованный доступ к API. Здесь должны быть:
/// - базовые заголовки (Auth)
/// - единая обработка ошибок
/// - ретраи/таймауты (по политике)
/// </summary>
public sealed class ApiClient
{
    private readonly HttpClient _http;

    public ApiClient(HttpClient http)
    {
        _http = http;
    }

    public async Task<bool> CheckHealthAsync(CancellationToken ct = default)
    {
        // Пример: /health -> { "status": "ok" }
        var dto = await _http.GetFromJsonAsync<HealthDto>("health", ct);
        return string.Equals(dto?.Status, "ok", StringComparison.OrdinalIgnoreCase);
    }
}


