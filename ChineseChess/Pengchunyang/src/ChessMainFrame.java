import java.awt.Container;
import java.awt.Font;
import java.awt.GridLayout;
import java.awt.event.MouseEvent;
import java.awt.event.MouseListener;

import javax.swing.Icon;
import javax.swing.ImageIcon;
import javax.swing.JButton;
import javax.swing.JFrame;
import javax.swing.JLabel;
import javax.swing.JToolBar;
import javax.swing.UIManager;


public class ChessMainFrame extends JFrame {
    
	//����
	JLabel chessBoard;	
	//������
	JToolBar toolBar;
	//��ʼ
	JButton start;
	//���
	JButton exit;
	//����
	Container con;
	//32������ͼ��
	JLabel[] chessIconArray = new JLabel[32];




	public ChessMainFrame(String title){
		
		con = this.getContentPane();
		con.setLayout(null);	   
		//����������
		toolBar = new JToolBar();
		//������ʼ�������ť
		start = new JButton("��ʼ");
		exit = new JButton("�˳�");

		//�������ӵ�������
		toolBar.setLayout(new GridLayout(0,2));
		toolBar.setBounds(0,0,558,30);
		toolBar.add(start);
		toolBar.add(exit);

		con.add(toolBar);

		//��ʼ������
		InitChesses();
		//��������ӵ�������
		for (int i=0;i<32;i++){
			con.add(chessIconArray[i]);
		}

		//�������̱�ǩ
		chessBoard = new JLabel(new ImageIcon("image\\Main.GIF"));
		chessBoard.setBounds(0,30,558,620);
		con.add(chessBoard);
		
		this.setTitle(title);
		this.setSize(558,670);
		this.setResizable(false);//���ڴ�С���ɸı�
		this.show();

	}

	/**
	 ** ��ʼ�������ϵ�����
	 */
	public void InitChesses(){
		int i,j;
		//ͼ��
		Icon chessIcon ;//���� Image���Ƶ�����ͼ��

		//��ʼ���ڳ���λ��   (24,56)   (480,56)
		chessIcon = new ImageIcon("image\\�ڳ�.GIF");//ͼƬ��55 ��55
		for (i=0,j=24;i<2;i++,j+=456){		
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,56,55,55);	
			chessIconArray[i].setName("��1");			
		}	

		//��
		chessIcon = new ImageIcon("image\\����.GIF");
		for (i=2,j=81;i<4;i++,j+=342){	
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,56,55,55);
			chessIconArray[i].setName("��1");
		}

		//��
		chessIcon = new ImageIcon("image\\����.GIF");
		for (i=4,j=138;i<6;i++,j+=228){	
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,56,55,55);
			chessIconArray[i].setName("��1");
		}

		//ʿ
		chessIcon = new ImageIcon("image\\��ʿ.GIF");
		for (i=6,j=195;i<8;i++,j+=114){
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,56,55,55);
			chessIconArray[i].setName("ʿ1");
		}

		//��
		chessIcon = new ImageIcon("image\\����.GIF");
		for (i=8,j=24;i<13;i++,j+=114){
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,227,55,55);
			chessIconArray[i].setName("��1" + i);
		}

		//��
		chessIcon = new ImageIcon("image\\����.GIF");			
		for (i=13,j=81;i<15;i++,j+=342){
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,170,55,55);
			chessIconArray[i].setName("��1" + i);
		}

		//��
		chessIcon = new ImageIcon("image\\�ڽ�.GIF");
		chessIconArray[15] = new JLabel(chessIcon);
		chessIconArray[15].setBounds(252,56,55,55);
		chessIconArray[15].setName("��1");


		//��
		chessIcon = new ImageIcon("image\\�쳵.GIF");
		for (i=16,j=24;i<18;i++,j+=456){
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,569,55,55);
			chessIconArray[i].setName("��2");
		}

		//��
		chessIcon = new ImageIcon("image\\����.GIF");
		for (i=18,j=81;i<20;i++,j+=342){
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,569,55,55);
			chessIconArray[i].setName("��2");
		}

		//��
		chessIcon = new ImageIcon("image\\����.GIF");			
		for (i=20,j=138;i<22;i++,j+=228){
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,569,55,55);
			chessIconArray[i].setName("��2");
		}

		//ʿ
		chessIcon = new ImageIcon("image\\��ʿ.GIF");
		for (i=22,j=195;i<24;i++,j+=114){
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,569,55,55);
			chessIconArray[i].setName("ʿ2");
		}

		//��
		chessIcon = new ImageIcon("image\\����.GIF");
		for (i=24,j=24;i<29;i++,j+=114){
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,398,55,55);
			chessIconArray[i].setName("��2" + i);
		}

		//��
		chessIcon = new ImageIcon("image\\����.GIF");
		for (i=29,j=81;i<31;i++,j+=342){
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,455,55,55);
			chessIconArray[i].setName("��2" + i);
		}

		//˧
		chessIcon = new ImageIcon("image\\�콫.GIF");			
		chessIconArray[31] = new JLabel(chessIcon);
		chessIconArray[31].setBounds(252,569,55,55);		
		chessIconArray[31].setName("˧2");
	}	
}
