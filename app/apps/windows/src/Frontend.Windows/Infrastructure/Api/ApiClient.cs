using System.Net.Http;
using System.Net.Http.Headers;
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
        var dto = await _http.GetFromJsonAsync<HealthDto>("api/v1/bff/desktop/health", ct);
        return string.Equals(dto?.Status, "ok", StringComparison.OrdinalIgnoreCase);
    }

    public async Task<LoginResponseDto?> LoginAsync(string username, string password, CancellationToken ct = default)
    {
        var response = await _http.PostAsJsonAsync(
            "api/v1/bff/desktop/auth/login",
            new LoginRequestDto
            {
                Username = username,
                Password = password
            },
            ct
        );
        response.EnsureSuccessStatusCode();

        var dto = await response.Content.ReadFromJsonAsync<LoginResponseDto>(cancellationToken: ct);
        if (!string.IsNullOrWhiteSpace(dto?.AccessToken))
        {
            _http.DefaultRequestHeaders.Authorization = new AuthenticationHeaderValue("Bearer", dto.AccessToken);
        }

        return dto;
    }

    public async Task<IReadOnlyList<TicketDto>> GetTicketsAsync(CancellationToken ct = default)
    {
        var items = await _http.GetFromJsonAsync<List<TicketDto>>("api/v1/bff/desktop/tickets", ct);
        return items ?? new List<TicketDto>();
    }

    public async Task LogoutAsync(CancellationToken ct = default)
    {
        if (_http.DefaultRequestHeaders.Authorization is not null)
        {
            await _http.PostAsync("api/v1/bff/desktop/auth/logout", content: null, ct);
        }

        _http.DefaultRequestHeaders.Authorization = null;
    }
}


