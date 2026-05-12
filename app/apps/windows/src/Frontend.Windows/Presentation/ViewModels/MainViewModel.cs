using System;
using System.Collections.ObjectModel;
using System.Net.Http;
using System.Threading.Tasks;
using Frontend.Windows.Domain.Dto;
using Frontend.Windows.Infrastructure.Api;
using Frontend.Windows.Infrastructure.Mvvm;
using Frontend.Windows.Infrastructure.Notifications;

namespace Frontend.Windows.Presentation.ViewModels;

public sealed class MainViewModel : ObservableObject
{
    private readonly ApiClient _api;
    private readonly NotificationService _notifications;

    private string _statusText = "Готово";
    private string _workspaceTitle = "Сотрудническая очередь";
    private string _workspaceSubtitle = "Desktop-клиент только для сотрудников сервисной службы.";
    private string _currentUserRole = "UNAUTHORIZED";
    private string _currentUserName = "Гость";
    private bool _isBusy;
    private bool _isAuthenticated;

    // --- Данные для управления тикетами ---
    private TicketDto? _selectedTicket;
    public ObservableCollection<TicketDto> Tickets { get; set; } = new();

    // --- Поля ввода: Логин ---
    private string? _loginEmail;
    private string? _loginPassword;

    public MainViewModel(ApiClient api, NotificationService notifications)
    {
        _api = api;
        _notifications = notifications;

        FooterText = "Desktop используется только сотрудниками: dispatcher, technician, manager, admin.";

        // Команды действий
        LoginCommand = new RelayCommand(LoginAsync);
        LogoutCommand = new RelayCommand(LogoutAsync);

        CheckApiCommand = new RelayCommand(async () =>
        {
            StatusText = "Проверка...";
            try
            {
                var ok = await _api.CheckHealthAsync();
                StatusText = ok ? "API доступен" : "API недоступен";
                _notifications.Info(StatusText);
            }
            catch (HttpRequestException ex)
            {
                StatusText = "Ошибка сети";
                _notifications.Error(ex.Message);
            }
        });
    }

    #region Properties

    public bool IsAuthenticated
    {
        get => _isAuthenticated;
        private set => SetProperty(ref _isAuthenticated, value);
    }

    public bool IsBusy
    {
        get => _isBusy;
        private set => SetProperty(ref _isBusy, value);
    }

    public string StatusText
    {
        get => _statusText;
        private set => SetProperty(ref _statusText, value);
    }

    public string WorkspaceTitle
    {
        get => _workspaceTitle;
        private set => SetProperty(ref _workspaceTitle, value);
    }

    public string WorkspaceSubtitle
    {
        get => _workspaceSubtitle;
        private set => SetProperty(ref _workspaceSubtitle, value);
    }

    public string CurrentUserRole
    {
        get => _currentUserRole;
        private set => SetProperty(ref _currentUserRole, value);
    }

    public string CurrentUserName
    {
        get => _currentUserName;
        private set => SetProperty(ref _currentUserName, value);
    }

    public string FooterText { get; }

    public bool IsLoginView => true;

    public TicketDto? SelectedTicket
    {
        get => _selectedTicket;
        set => SetProperty(ref _selectedTicket, value);
    }

    // Auth Fields
    public string? LoginEmail { get => _loginEmail; set => SetProperty(ref _loginEmail, value); }
    public string? LoginPassword { get => _loginPassword; set => SetProperty(ref _loginPassword, value); }

    #endregion

    #region Commands
    public RelayCommand LoginCommand { get; }
    public RelayCommand LogoutCommand { get; }
    public RelayCommand CheckApiCommand { get; }
    #endregion

    #region Logic Methods

    private async Task LoginAsync()
    {
        if (IsBusy) return;

        var email = (LoginEmail ?? "").Trim();
        var password = LoginPassword ?? "";

        if (string.IsNullOrWhiteSpace(email) || string.IsNullOrWhiteSpace(password))
        {
            StatusText = "Введите логин и пароль";
            _notifications.Info(StatusText);
            return;
        }

        try
        {
            IsBusy = true;
            StatusText = "Вход...";
            var login = await _api.LoginAsync(email, password);
            var role = login?.User?.Role?.Trim().ToUpperInvariant() ?? string.Empty;

            if (!IsEmployeeRole(role))
            {
                await _api.LogoutAsync();
                IsAuthenticated = false;
                StatusText = "Desktop доступен только сотрудникам";
                _notifications.Info("Клиентские учетные записи должны использовать web/mobile. Desktop предназначен только для сотрудников.");
                return;
            }

            var tickets = await _api.GetTicketsAsync();
            Tickets.Clear();
            foreach (var item in tickets)
            {
                Tickets.Add(item);
            }

            CurrentUserRole = role;
            CurrentUserName = login?.User?.Name ?? email;
            WorkspaceTitle = role switch
            {
                "TECHNICIAN" => "Рабочее место техника",
                "DISPATCHER" => "Панель диспетчера",
                "MANAGER" => "Панель руководителя",
                _ => "Административная сотрудническая панель"
            };
            WorkspaceSubtitle = "Клиентские сервисы скрыты. Здесь только операционная работа сотрудников.";
            IsAuthenticated = true;
            StatusText = $"Вход выполнен: {role}";
            _notifications.Info(StatusText);
        }
        catch (Exception ex)
        {
            StatusText = "Ошибка входа";
            _notifications.Error(ex.Message);
        }
        finally { IsBusy = false; }
    }

    private async Task LogoutAsync()
    {
        try
        {
            await _api.LogoutAsync();
        }
        catch
        {
            // Local sign-out should still succeed if the server session is already gone.
        }

        IsAuthenticated = false;
        Tickets.Clear();
        LoginPassword = "";
        CurrentUserRole = "UNAUTHORIZED";
        CurrentUserName = "Гость";
        WorkspaceTitle = "Сотрудническая очередь";
        WorkspaceSubtitle = "Desktop-клиент только для сотрудников сервисной службы.";
        StatusText = "Выход выполнен";
    }

    #endregion

    private static bool IsEmployeeRole(string role)
    {
        return role is "ADMIN" or "MANAGER" or "DISPATCHER" or "TECHNICIAN";
    }
}