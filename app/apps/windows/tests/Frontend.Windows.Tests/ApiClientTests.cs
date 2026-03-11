using System.Net;
using System.Net.Http;
using System.Text;
using Frontend.Windows.Infrastructure.Api;
using Xunit;

namespace Frontend.Windows.Tests;

public sealed class ApiClientTests
{
    [Fact]
    public async Task CheckHealthAsync_ReturnsTrue_WhenStatusOk()
    {
        var handler = new StubHandler(_ => new HttpResponseMessage(HttpStatusCode.OK)
        {
            Content = new StringContent("{\"status\":\"ok\"}", Encoding.UTF8, "application/json")
        });

        var http = new HttpClient(handler) { BaseAddress = new Uri("https://example.corp/api/") };
        var client = new ApiClient(http);

        var ok = await client.CheckHealthAsync();
        Assert.True(ok);
    }

    private sealed class StubHandler : HttpMessageHandler
    {
        private readonly Func<HttpRequestMessage, HttpResponseMessage> _handler;

        public StubHandler(Func<HttpRequestMessage, HttpResponseMessage> handler)
        {
            _handler = handler;
        }

        protected override Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken)
        {
            return Task.FromResult(_handler(request));
        }
    }
}


