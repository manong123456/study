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
    
	//棋盘
	JLabel chessBoard;	
	//工具栏
	JToolBar toolBar;
	//开始
	JButton start;
	//提出
	JButton exit;
	//容器
	Container con;
	//32个棋子图标
	JLabel[] chessIconArray = new JLabel[32];




	public ChessMainFrame(String title){
		
		con = this.getContentPane();
		con.setLayout(null);	   
		//创建工具栏
		toolBar = new JToolBar();
		//创建开始和提出按钮
		start = new JButton("开始");
		exit = new JButton("退出");

		//把组件添加到工具栏
		toolBar.setLayout(new GridLayout(0,2));
		toolBar.setBounds(0,0,558,30);
		toolBar.add(start);
		toolBar.add(exit);

		con.add(toolBar);

		//初始化棋子
		InitChesses();
		//将棋子添加到容器上
		for (int i=0;i<32;i++){
			con.add(chessIconArray[i]);
		}

		//创建棋盘标签
		chessBoard = new JLabel(new ImageIcon("image\\Main.GIF"));
		chessBoard.setBounds(0,30,558,620);
		con.add(chessBoard);
		
		this.setTitle(title);
		this.setSize(558,670);
		this.setResizable(false);//窗口大小不可改变
		this.show();

	}

	/**
	 ** 初始化棋盘上的棋子
	 */
	public void InitChesses(){
		int i,j;
		//图标
		Icon chessIcon ;//根据 Image绘制的棋子图标

		//初始化黑车的位置   (24,56)   (480,56)
		chessIcon = new ImageIcon("image\\黑车.GIF");//图片宽55 高55
		for (i=0,j=24;i<2;i++,j+=456){		
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,56,55,55);	
			chessIconArray[i].setName("车1");			
		}	

		//马
		chessIcon = new ImageIcon("image\\黑马.GIF");
		for (i=2,j=81;i<4;i++,j+=342){	
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,56,55,55);
			chessIconArray[i].setName("马1");
		}

		//相
		chessIcon = new ImageIcon("image\\黑象.GIF");
		for (i=4,j=138;i<6;i++,j+=228){	
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,56,55,55);
			chessIconArray[i].setName("象1");
		}

		//士
		chessIcon = new ImageIcon("image\\黑士.GIF");
		for (i=6,j=195;i<8;i++,j+=114){
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,56,55,55);
			chessIconArray[i].setName("士1");
		}

		//卒
		chessIcon = new ImageIcon("image\\黑卒.GIF");
		for (i=8,j=24;i<13;i++,j+=114){
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,227,55,55);
			chessIconArray[i].setName("卒1" + i);
		}

		//炮
		chessIcon = new ImageIcon("image\\黑炮.GIF");			
		for (i=13,j=81;i<15;i++,j+=342){
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,170,55,55);
			chessIconArray[i].setName("炮1" + i);
		}

		//将
		chessIcon = new ImageIcon("image\\黑将.GIF");
		chessIconArray[15] = new JLabel(chessIcon);
		chessIconArray[15].setBounds(252,56,55,55);
		chessIconArray[15].setName("将1");


		//车
		chessIcon = new ImageIcon("image\\红车.GIF");
		for (i=16,j=24;i<18;i++,j+=456){
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,569,55,55);
			chessIconArray[i].setName("车2");
		}

		//马
		chessIcon = new ImageIcon("image\\红马.GIF");
		for (i=18,j=81;i<20;i++,j+=342){
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,569,55,55);
			chessIconArray[i].setName("马2");
		}

		//相
		chessIcon = new ImageIcon("image\\红象.GIF");			
		for (i=20,j=138;i<22;i++,j+=228){
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,569,55,55);
			chessIconArray[i].setName("象2");
		}

		//士
		chessIcon = new ImageIcon("image\\红士.GIF");
		for (i=22,j=195;i<24;i++,j+=114){
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,569,55,55);
			chessIconArray[i].setName("士2");
		}

		//兵
		chessIcon = new ImageIcon("image\\红卒.GIF");
		for (i=24,j=24;i<29;i++,j+=114){
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,398,55,55);
			chessIconArray[i].setName("卒2" + i);
		}

		//炮
		chessIcon = new ImageIcon("image\\红炮.GIF");
		for (i=29,j=81;i<31;i++,j+=342){
			chessIconArray[i] = new JLabel(chessIcon);
			chessIconArray[i].setBounds(j,455,55,55);
			chessIconArray[i].setName("炮2" + i);
		}

		//帅
		chessIcon = new ImageIcon("image\\红将.GIF");			
		chessIconArray[31] = new JLabel(chessIcon);
		chessIconArray[31].setBounds(252,569,55,55);		
		chessIconArray[31].setName("帅2");
	}	
}
