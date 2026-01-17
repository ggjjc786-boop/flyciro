"""
闲鱼专属 API 调用示例 (Python 版本)

本文件提供了调用闲鱼专属 API 的完整示例代码
适用于 Python 环境下的外部软件集成

依赖:
    pip install requests
"""

import requests
import time
from typing import Dict, List, Optional

API_BASE_URL = 'http://localhost:3000'


class XianyuAPI:
    """闲鱼专属 API 客户端"""
    
    def __init__(self, base_url: str = API_BASE_URL):
        self.base_url = base_url
        self.session = requests.Session()
        self.session.headers.update({
            'Content-Type': 'application/json'
        })
    
    def submit_account_email_info(self, account_data: Dict) -> Dict:
        """
        提交账号的 Outlook 邮箱信息到服务器
        
        Args:
            account_data: 账号数据字典，包含以下字段:
                - username: 账号用户名
                - password: 账号密码
                - outlook_email: Outlook 邮箱地址
                - mail_client_id: OAuth Client ID
                - mail_refresh_token: OAuth Refresh Token
        
        Returns:
            包含 success 和 account_id 或 error 的字典
        """
        try:
            response = self.session.post(
                f'{self.base_url}/api/xianyu/submit-email-info',
                json=account_data
            )
            response.raise_for_status()
            result = response.json()
            
            if result.get('success'):
                print(f'✅ 邮箱信息提交成功')
                print(f'账号ID: {result.get("account_id")}')
                return {
                    'success': True,
                    'account_id': result.get('account_id')
                }
            else:
                error_msg = result.get('message', '未知错误')
                print(f'❌ 提交失败: {error_msg}')
                return {
                    'success': False,
                    'error': error_msg
                }
                
        except requests.exceptions.RequestException as e:
            print(f'❌ 网络错误: {str(e)}')
            return {
                'success': False,
                'error': str(e)
            }
    
    def batch_submit_accounts(self, accounts: List[Dict], delay: float = 0.5) -> Dict:
        """
        批量提交多个账号的邮箱信息
        
        Args:
            accounts: 账号数据列表
            delay: 每次提交之间的延迟（秒）
        
        Returns:
            包含成功、失败统计的字典
        """
        print(f'开始批量提交 {len(accounts)} 个账号的邮箱信息...\n')
        
        results = {
            'success': 0,
            'failed': 0,
            'errors': []
        }
        
        for i, account in enumerate(accounts, 1):
            username = account.get('username', '未知')
            print(f'[{i}/{len(accounts)}] 提交账号: {username}')
            
            result = self.submit_account_email_info(account)
            
            if result['success']:
                results['success'] += 1
            else:
                results['failed'] += 1
                results['errors'].append({
                    'username': username,
                    'error': result.get('error', '未知错误')
                })
            
            # 避免请求过快
            if i < len(accounts):
                time.sleep(delay)
        
        print('\n============ 批量提交完成 ============')
        print(f'✅ 成功: {results["success"]}')
        print(f'❌ 失败: {results["failed"]}')
        
        if results['errors']:
            print('\n失败详情:')
            for err in results['errors']:
                print(f'  - {err["username"]}: {err["error"]}')
        
        return results
    
    def get_account_id(self, username: str) -> Optional[int]:
        """
        根据账号用户名获取账号ID
        
        Args:
            username: 账号用户名
        
        Returns:
            账号ID，如果失败则返回 None
        """
        try:
            response = self.session.post(
                f'{self.base_url}/api/xianyu/get-account-id',
                json={'username': username}
            )
            response.raise_for_status()
            result = response.json()
            
            if result.get('success'):
                return result.get('account_id')
            else:
                print(f'❌ 获取账号ID失败: {result.get("message")}')
                return None
                
        except requests.exceptions.RequestException as e:
            print(f'❌ 网络错误: {str(e)}')
            return None
    
    def get_email_verification_code(self, account_id: int) -> Optional[str]:
        """
        获取账号的邮箱验证码
        
        Args:
            account_id: 账号ID
        
        Returns:
            验证码，如果失败则返回 None
        """
        try:
            response = self.session.post(
                f'{self.base_url}/api/xianyu/get-email-code',
                json={'account_id': account_id}
            )
            response.raise_for_status()
            result = response.json()
            
            if result.get('success'):
                code = result.get('code')
                email = result.get('email')
                print(f'✅ 验证码获取成功: {code}')
                print(f'邮箱: {email}')
                return code
            else:
                error_msg = result.get('message', '未知错误')
                print(f'❌ 获取失败: {error_msg}')
                return None
                
        except requests.exceptions.RequestException as e:
            print(f'❌ 网络错误: {str(e)}')
            return None


def main():
    """主函数 - 演示 API 使用"""
    print('========== 闲鱼专属 API 测试 (Python) ==========\n')
    
    # 创建 API 客户端
    api = XianyuAPI()
    
    # 示例1: 提交单个账号
    print('示例1: 提交单个账号的邮箱信息')
    single_account = {
        'username': 'test@example.com',
        'password': 'password123',
        'outlook_email': 'test@outlook.com',
        'mail_client_id': 'your-client-id-here',
        'mail_refresh_token': 'your-refresh-token-here'
    }
    
    result = api.submit_account_email_info(single_account)
    print(f'提交结果: {result}\n')
    
    # 示例2: 批量提交多个账号
    print('\n示例2: 批量提交多个账号')
    accounts = [
        {
            'username': 'user1@example.com',
            'password': 'pass1',
            'outlook_email': 'user1@outlook.com',
            'mail_client_id': 'client-id-1',
            'mail_refresh_token': 'token-1'
        },
        {
            'username': 'user2@example.com',
            'password': 'pass2',
            'outlook_email': 'user2@outlook.com',
            'mail_client_id': 'client-id-2',
            'mail_refresh_token': 'token-2'
        }
    ]
    
    batch_results = api.batch_submit_accounts(accounts)
    
    # 示例3: 测试获取验证码（需要先提交过邮箱信息）
    print('\n\n示例3: 测试获取邮箱验证码')
    account_id = api.get_account_id('test@example.com')
    if account_id:
        code = api.get_email_verification_code(account_id)
        print(f'最终验证码: {code}')


def example_read_from_csv():
    """示例: 从 CSV 文件读取账号信息并批量提交"""
    import csv
    
    print('示例: 从 CSV 文件读取账号信息\n')
    
    # CSV 文件格式示例:
    # username,password,outlook_email,mail_client_id,mail_refresh_token
    # user1@example.com,pass1,user1@outlook.com,client-id-1,token-1
    # user2@example.com,pass2,user2@outlook.com,client-id-2,token-2
    
    csv_file = 'accounts.csv'
    
    try:
        accounts = []
        with open(csv_file, 'r', encoding='utf-8') as f:
            reader = csv.DictReader(f)
            for row in reader:
                accounts.append({
                    'username': row['username'],
                    'password': row['password'],
                    'outlook_email': row['outlook_email'],
                    'mail_client_id': row['mail_client_id'],
                    'mail_refresh_token': row['mail_refresh_token']
                })
        
        print(f'从 {csv_file} 读取了 {len(accounts)} 个账号')
        
        api = XianyuAPI()
        results = api.batch_submit_accounts(accounts)
        
        print(f'\n✅ 成功提交 {results["success"]} 个账号')
        print(f'❌ 失败 {results["failed"]} 个账号')
        
    except FileNotFoundError:
        print(f'❌ 文件 {csv_file} 不存在')
    except Exception as e:
        print(f'❌ 读取文件时出错: {str(e)}')


def example_read_from_json():
    """示例: 从 JSON 文件读取账号信息并批量提交"""
    import json
    
    print('示例: 从 JSON 文件读取账号信息\n')
    
    # JSON 文件格式示例:
    # [
    #   {
    #     "username": "user1@example.com",
    #     "password": "pass1",
    #     "outlook_email": "user1@outlook.com",
    #     "mail_client_id": "client-id-1",
    #     "mail_refresh_token": "token-1"
    #   }
    # ]
    
    json_file = 'accounts.json'
    
    try:
        with open(json_file, 'r', encoding='utf-8') as f:
            accounts = json.load(f)
        
        print(f'从 {json_file} 读取了 {len(accounts)} 个账号')
        
        api = XianyuAPI()
        results = api.batch_submit_accounts(accounts)
        
        print(f'\n✅ 成功提交 {results["success"]} 个账号')
        print(f'❌ 失败 {results["failed"]} 个账号')
        
    except FileNotFoundError:
        print(f'❌ 文件 {json_file} 不存在')
    except Exception as e:
        print(f'❌ 读取文件时出错: {str(e)}')


if __name__ == '__main__':
    # 运行基本示例
    main()
    
    # 取消注释以下代码来测试从文件读取
    # example_read_from_csv()
    # example_read_from_json()
