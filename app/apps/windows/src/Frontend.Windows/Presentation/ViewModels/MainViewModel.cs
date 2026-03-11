using System;
using System.Net.Http;
using System.Threading.Tasks;
using Frontend.Windows.Infrastructure.Api;
using Frontend.Windows.Infrastructure.Mvvm;
using Frontend.Windows.Infrastructure.Notifications;

namespace Frontend.Windows.Presentation.ViewModels;

public sealed class MainViewModel : ObservableObject
{
    private readonly ApiClient _api;
    private readonly NotificationService _notifications;

    private string _statusText = "Готово";

    // --- auth view state ---
    private bool _isLoginView = true;
    private bool _isRegisterView;
    private bool _isRecoverView;

    private bool _isBusy;

    // --- login ---
    private string? _loginEmail;
    private string? _loginPassword;

    // --- register ---
    private string? _registerEmail;
    private string? _registerPassword;
    private string? _registerPasswordConfirm;

    // --- recover ---
    private string? _recoverEmail;

    public MainViewModel(ApiClient api, NotificationService notifications)
    {
        _api = api;
        _notifications = notifications;

        FooterText = "IDEF0: A0 UI, A1 данные, A2 команды, A3 процессы, A4 уведомления";

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
                StatusText = "Ошибка сети/сертификата";
                _notifications.Error(ex.Message);
            }
        });

        // Переключение экранов (тоже RelayCommand(Func<Task>))
        ShowLoginCommand = new RelayCommand(() =>
        {
            SetAuthMode(AuthMode.Login);
            return Task.CompletedTask;
        });

        ShowRegisterCommand = new RelayCommand(() =>
        {
            SetAuthMode(AuthMode.Register);
            return Task.CompletedTask;
        });

        ShowRecoverCommand = new RelayCommand(() =>
        {
            SetAuthMode(AuthMode.Recover);
            return Task.CompletedTask;
        });

        // Действия
        LoginCommand = new RelayCommand(LoginAsync);
        RegisterCommand = new RelayCommand(RegisterAsync);
        RecoverCommand = new RelayCommand(RecoverAsync);

        SetAuthMode(AuthMode.Login);
    }

    // ----- existing -----
    public string StatusText
    {
        get => _statusText;
        private set => SetProperty(ref _statusText, value);
    }

    public string FooterText { get; }

    public RelayCommand CheckApiCommand { get; }

    // ----- new: commands -----
    public RelayCommand ShowLoginCommand { get; }
    public RelayCommand ShowRegisterCommand { get; }
    public RelayCommand ShowRecoverCommand { get; }

    public RelayCommand LoginCommand { get; }
    public RelayCommand RegisterCommand { get; }
    public RelayCommand RecoverCommand { get; }

    // ----- new: flags -----
    public bool IsLoginView
    {
        get => _isLoginView;
        private set => SetProperty(ref _isLoginView, value);
    }

    public bool IsRegisterView
    {
        get => _isRegisterView;
        private set => SetProperty(ref _isRegisterView, value);
    }

    public bool IsRecoverView
    {
        get => _isRecoverView;
        private set => SetProperty(ref _isRecoverView, value);
    }

    public bool IsBusy
    {
        get => _isBusy;
        private set => SetProperty(ref _isBusy, value);
    }

    // ----- new: fields -----
    public string? LoginEmail
    {
        get => _loginEmail;
        set => SetProperty(ref _loginEmail, value);
    }

    // приходит из PasswordBox в MainWindow.xaml.cs
    public string? LoginPassword
    {
        get => _loginPassword;
        set => SetProperty(ref _loginPassword, value);
    }

    public string? RegisterEmail
    {
        get => _registerEmail;
        set => SetProperty(ref _registerEmail, value);
    }

    public string? RegisterPassword
    {
        get => _registerPassword;
        set => SetProperty(ref _registerPassword, value);
    }

    public string? RegisterPasswordConfirm
    {
        get => _registerPasswordConfirm;
        set => SetProperty(ref _registerPasswordConfirm, value);
    }

    public string? RecoverEmail
    {
        get => _recoverEmail;
        set => SetProperty(ref _recoverEmail, value);
    }

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

        if (string.IsNullOrWhiteSpace(email))
        {
            StatusText = "Введите email";
            _notifications.Info(StatusText);
            return;
        }

        if (string.IsNullOrWhiteSpace(password))
        {
            StatusText = "Введите пароль";
            _notifications.Info(StatusText);
            return;
        }

        try
        {
            IsBusy = true;
            StatusText = "Вход...";

            // TODO: подключить реальный метод API, когда появится эндпоинт
            await Task.Delay(400);

            StatusText = "Успешный вход (заглушка)";
            _notifications.Info(StatusText);
        }
        catch (Exception ex)
        {
            StatusText = "Ошибка входа";
            _notifications.Error(ex.Message);
        }
        finally
        {
            IsBusy = false;
        }
    }

    private async Task RegisterAsync()
    {
        if (IsBusy) return;

        var email = (RegisterEmail ?? "").Trim();
        var password = RegisterPassword ?? "";
        var confirm = RegisterPasswordConfirm ?? "";

        if (string.IsNullOrWhiteSpace(email))
        {
            StatusText = "Введите email";
            _notifications.Info(StatusText);
            return;
        }

        if (string.IsNullOrWhiteSpace(password))
        {
            StatusText = "Введите пароль";
            _notifications.Info(StatusText);
            return;
        }

        if (password.Length < 6)
        {
            StatusText = "Пароль должен быть не короче 6 символов";
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

            // TODO: подключить реальный метод API
            await Task.Delay(500);

            StatusText = "Регистрация успешна (заглушка)";
            _notifications.Info(StatusText);

            SetAuthMode(AuthMode.Login);
        }
        catch (Exception ex)
        {
            StatusText = "Ошибка регистрации";
            _notifications.Error(ex.Message);
        }
        finally
        {
            IsBusy = false;
        }
    }

    private async Task RecoverAsync()
    {
        if (IsBusy) return;

        var email = (RecoverEmail ?? "").Trim();

        if (string.IsNullOrWhiteSpace(email))
        {
            StatusText = "Введите email";
            _notifications.Info(StatusText);
            return;
        }

        try
        {
            IsBusy = true;
            StatusText = "Отправка инструкции...";

            // TODO: подключить реальный метод API
            await Task.Delay(450);

            StatusText = "Инструкция отправлена (заглушка)";
            _notifications.Info(StatusText);

            SetAuthMode(AuthMode.Login);
        }
        catch (Exception ex)
        {
            StatusText = "Ошибка восстановления";
            _notifications.Error(ex.Message);
        }
        finally
        {
            IsBusy = false;
        }
    }
}
