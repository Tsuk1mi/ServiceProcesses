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
    private bool _isBusy;
    private bool _isAuthenticated;

    // --- Состояния экранов авторизации ---
    private bool _isLoginView = true;
    private bool _isRegisterView;
    private bool _isRecoverView;

    // --- Данные для управления тикетами ---
    private TicketDto? _selectedTicket;
    public ObservableCollection<TicketDto> Tickets { get; set; } = new();

    // --- Поля ввода: Логин ---
    private string? _loginEmail;
    private string? _loginPassword;

    // --- Поля ввода: Регистрация ---
    private string? _registerEmail;
    private string? _registerPassword;
    private string? _registerPasswordConfirm;

    // --- Поля ввода: Восстановление ---
    private string? _recoverEmail;

    public MainViewModel(ApiClient api, NotificationService notifications)
    {
        _api = api;
        _notifications = notifications;

        FooterText = "IDEF0: A0 UI, A1 данные, A2 команды, A3 процессы, A4 уведомления";

        // Команды навигации
        ShowLoginCommand = new RelayCommand(() => { SetAuthMode(AuthMode.Login); return Task.CompletedTask; });
        ShowRegisterCommand = new RelayCommand(() => { SetAuthMode(AuthMode.Register); return Task.CompletedTask; });
        ShowRecoverCommand = new RelayCommand(() => { SetAuthMode(AuthMode.Recover); return Task.CompletedTask; });

        // Команды действий
        LoginCommand = new RelayCommand(LoginAsync);
        RegisterCommand = new RelayCommand(RegisterAsync);
        RecoverCommand = new RelayCommand(RecoverAsync);
        LogoutCommand = new RelayCommand(Logout);

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

    public string FooterText { get; }

    public bool IsLoginView { get => _isLoginView; private set => SetProperty(ref _isLoginView, value); }
    public bool IsRegisterView { get => _isRegisterView; private set => SetProperty(ref _isRegisterView, value); }
    public bool IsRecoverView { get => _isRecoverView; private set => SetProperty(ref _isRecoverView, value); }

    public TicketDto? SelectedTicket
    {
        get => _selectedTicket;
        set => SetProperty(ref _selectedTicket, value);
    }

    // Auth Fields
    public string? LoginEmail { get => _loginEmail; set => SetProperty(ref _loginEmail, value); }
    public string? LoginPassword { get => _loginPassword; set => SetProperty(ref _loginPassword, value); }
    public string? RegisterEmail { get => _registerEmail; set => SetProperty(ref _registerEmail, value); }
    public string? RegisterPassword { get => _registerPassword; set => SetProperty(ref _registerPassword, value); }
    public string? RegisterPasswordConfirm { get => _registerPasswordConfirm; set => SetProperty(ref _registerPasswordConfirm, value); }
    public string? RecoverEmail { get => _recoverEmail; set => SetProperty(ref _recoverEmail, value); }

    #endregion

    #region Commands
    public RelayCommand ShowLoginCommand { get; }
    public RelayCommand ShowRegisterCommand { get; }
    public RelayCommand ShowRecoverCommand { get; }
    public RelayCommand LoginCommand { get; }
    public RelayCommand RegisterCommand { get; }
    public RelayCommand RecoverCommand { get; }
    public RelayCommand LogoutCommand { get; }
    public RelayCommand CheckApiCommand { get; }
    #endregion

    #region Logic Methods

    private enum AuthMode { Login, Register, Recover }

    private void SetAuthMode(AuthMode mode)
    {
        IsLoginView = mode == AuthMode.Login;
        IsRegisterView = mode == AuthMode.Register;
        IsRecoverView = mode == AuthMode.Recover;
        StatusText = "Готово";
    }

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
            await Task.Delay(800); 

            LoadMockData(); // Загружаем данные для макета
            IsAuthenticated = true;
            _notifications.Info("Вход выполнен");
        }
        catch (Exception ex)
        {
            StatusText = "Ошибка входа";
            _notifications.Error(ex.Message);
        }
        finally { IsBusy = false; }
    }

    private async Task RegisterAsync()
    {
        if (IsBusy) return;

        var email = (RegisterEmail ?? "").Trim();
        var password = RegisterPassword ?? "";
        var confirm = RegisterPasswordConfirm ?? "";

        if (string.IsNullOrWhiteSpace(email) || string.IsNullOrWhiteSpace(password))
        {
            StatusText = "Заполните поля регистрации";
            _notifications.Info(StatusText);
            return;
        }

        if (!string.Equals(password, confirm, StringComparison.Ordinal))
        {
            StatusText = "Пароли не совпадают";
            _notifications.Info(StatusText);
            return;
        }

        try
        {
            IsBusy = true;
            StatusText = "Регистрация...";
            await Task.Delay(1000);

            StatusText = "Регистрация успешна";
            _notifications.Info(StatusText);
            SetAuthMode(AuthMode.Login);
        }
        catch (Exception ex)
        {
            StatusText = "Ошибка регистрации";
            _notifications.Error(ex.Message);
        }
        finally { IsBusy = false; }
    }

    private async Task RecoverAsync()
    {
        if (IsBusy) return;
        if (string.IsNullOrWhiteSpace(RecoverEmail)) { _notifications.Info("Введите email"); return; }

        try
        {
            IsBusy = true;
            StatusText = "Восстановление...";
            await Task.Delay(500);
            StatusText = "Инструкция отправлена";
            _notifications.Info(StatusText);
            SetAuthMode(AuthMode.Login);
        }
        catch (Exception ex) { _notifications.Error(ex.Message); }
        finally { IsBusy = false; }
    }

    private void LoadMockData()
    {
        Tickets.Clear();
        Tickets.Add(new TicketDto { Id = "#2313", ObjectName = "Лифт чинить - москва сити", Status = "выполнен", Priority = "высокий", AssignedTo = "Ваня" });
        Tickets.Add(new TicketDto { Id = "#1231", ObjectName = "Лифт чинить - москва сити", Status = "выполнен", Priority = "высокий", AssignedTo = "Леша" });
        Tickets.Add(new TicketDto { Id = "#1232", ObjectName = "Замена ламп - башня федерация", Status = "в процессе", Priority = "средний", AssignedTo = "Ваня" });
    }

    private Task Logout()
    {
        IsAuthenticated = false;
        Tickets.Clear();
        LoginPassword = "";
        return Task.CompletedTask;
    }

    #endregion
}