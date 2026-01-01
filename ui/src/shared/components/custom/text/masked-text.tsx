import { ReactElement, useCallback, useState } from 'react';

export type MaskedTextType = 'email' | 'phone' | 'text' | 'secret';

export interface MaskedTextProps {
  text: string;
  type: MaskedTextType;
  className?: string;
  clickable?: boolean; // 新增参数：是否支持双击显示/隐藏和单击复制
  onRevealedClick?: () => void; // 显示状态下的点击回调
}

/**
 * 隐藏中间部分文字的组件
 * @param text - 要处理的文本
 * @param type - 处理类型：邮箱、手机号、文本、密钥
 * @param className - 额外的CSS类名
 * @param clickable - 是否支持双击显示/隐藏和单击复制功能
 * @param onRevealedClick - 显示状态下的点击回调
 */
export function MaskedText({ text, type, className, clickable = false, onRevealedClick }: MaskedTextProps): ReactElement {
  const [isRevealed, setIsRevealed] = useState(false);

  const maskText = (text: string, type: MaskedTextType): string => {
    if (!text) {
      return '';
    }

    switch (type) {
      case 'email':
        return maskEmail(text);
      case 'phone':
        return maskPhone(text);
      case 'text':
        return maskGenericText(text);
      case 'secret':
        return maskSecret(text);
      default:
        return text;
    }
  };

  const maskEmail = (email: string): string => {
    // 邮箱：隐藏前面2个字符之后到@之前的字符
    const atIndex = email.indexOf('@');
    if (atIndex <= 2) {
      // 如果@符号在前2个字符内或不存在，直接返回原文本
      return email;
    }

    const prefix = email.substring(0, 2);
    const suffix = email.substring(atIndex);
    const hiddenPart = '*'.repeat(atIndex - 2);

    return prefix + hiddenPart + suffix;
  };

  const maskPhone = (phone: string): string => {
    // 手机号：隐藏中间4位
    if (phone.length <= 7) {
      // 如果长度不足8位，无法隐藏中间4位，返回原文本
      return phone;
    }

    const start = phone.substring(0, 3);
    const end = phone.substring(phone.length - 4);

    return start + '****' + end;
  };

  const maskGenericText = (text: string): string => {
    // 文本：头尾保留3个，中间用***代替。不足6个头尾平均分开显示
    if (text.length === 0) {
      return '';
    }

    if (text.length === 1) {
      return text;
    }

    if (text.length <= 6) {
      // 不足6个字符时，头尾平均分配
      const frontCount = Math.ceil(text.length / 2);
      const backCount = text.length - frontCount;

      if (backCount === 0) {
        return text;
      }

      const front = text.substring(0, frontCount);
      const back = text.substring(text.length - backCount);

      return front + '***' + back;
    }

    // 6个字符以上，头尾各保留3个
    const front = text.substring(0, 3);
    const back = text.substring(text.length - 3);

    return front + '***' + back;
  };

  const maskSecret = (secret: string): string => {
    // 密钥：根据长度自适应显示更多字符
    // 短密钥（<= 16）: 头4尾4
    // 中等密钥（17-32）: 头6尾6
    // 长密钥（> 32）: 头8尾8
    if (secret.length === 0) {
      return '';
    }

    if (secret.length <= 8) {
      // 非常短的密钥，头尾各显示2个
      const frontCount = Math.min(2, Math.ceil(secret.length / 2));
      const backCount = Math.min(2, secret.length - frontCount);

      if (backCount === 0) {
        return secret;
      }

      const front = secret.substring(0, frontCount);
      const back = secret.substring(secret.length - backCount);

      return front + '***' + back;
    }

    if (secret.length <= 16) {
      // 短密钥：头4尾4
      const front = secret.substring(0, 4);
      const back = secret.substring(secret.length - 4);
      return front + '***' + back;
    }

    if (secret.length <= 32) {
      // 中等密钥：头6尾6
      const front = secret.substring(0, 6);
      const back = secret.substring(secret.length - 6);
      return front + '***' + back;
    }

    // 长密钥：头8尾8
    const front = secret.substring(0, 8);
    const back = secret.substring(secret.length - 8);
    return front + '***' + back;
  };

  // 处理单击事件 - 显示时调用回调
  const handleClick = useCallback(() => {
    if (!clickable || !isRevealed || !onRevealedClick) return;
    onRevealedClick();
  }, [clickable, isRevealed, onRevealedClick]);

  // 处理双击事件 - 切换显示/隐藏
  const handleDoubleClick = useCallback(() => {
    if (!clickable) return;
    setIsRevealed(!isRevealed);
  }, [clickable, isRevealed]);

  // 显示的文本内容
  const displayText = clickable && isRevealed ? text : maskText(text, type);

  // 获取提示文本
  const getTooltipText = () => {
    if (!clickable) return undefined;
    if (isRevealed) {
      return '双击隐藏，单击执行操作';
    }
    return '双击显示';
  };

  return (
    <span
      className={clickable ? `cursor-pointer select-none ${className || ''}` : className}
      onClick={handleClick}
      onDoubleClick={handleDoubleClick}
      title={getTooltipText()}
    >
      {displayText}
    </span>
  );
}

export default MaskedText;
