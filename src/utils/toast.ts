import toast from 'react-hot-toast';

export const showSuccessToast = (message: string) => {
  return toast.success(message, {
    duration: 4000,
    position: 'top-right',
  });
};

export const showErrorToast = (message: string) => {
  return toast.error(message, {
    duration: 4000,
    position: 'top-right',
  });
};

export const showInfoToast = (message: string) => {
  return toast(message, {
    duration: 3000,
    position: 'top-right',
    icon: 'ℹ️',
  });
};

export const showLoadingToast = (message: string) => {
  return toast.loading(message, {
    position: 'top-right',
  });
};

export const dismissToast = (toastId: string) => {
  toast.dismiss(toastId);
};

export const dismissAllToasts = () => {
  toast.dismiss();
};
